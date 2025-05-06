use mesh_loader::collada::from_str;
use bevy::prelude::*;
use bevy::render::mesh::{ Indices, Mesh };
use bevy::render::render_resource::PrimitiveTopology;
use mesh_loader::Scene;
use std::io::Cursor;
use bevy::{ asset::{ io::Reader, AssetLoader, LoadContext }, prelude::* };
use thiserror::Error;

#[derive(Error, Debug)]
enum DaeError {
    #[error("Failed to load STL")] Io(#[from] std::io::Error),
}

pub struct ColladaPlugin;
impl Plugin for ColladaPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<DaeLoader>();
    }
}

#[derive(Default)]
struct DaeLoader;
impl AssetLoader for DaeLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = DaeError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        #[allow(unused_variables)] load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let collada_str = std::str
            ::from_utf8(&bytes)
            .map_err(|e| DaeError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        let collada = from_str(collada_str).unwrap();

        Ok(dae_to_triangle_mesh(&collada, 0).unwrap())
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dae"];
        EXTENSIONS
    }
}

// fn main() {
//     let collada_str = include_str!("../assets/models/joint.dae");
//     let collada = from_str(collada_str).unwrap();
//     println!("{:#?}", collada.meshes);

//     // Example usage of the new function
//     if let Some(_triangle_mesh) = dae_to_triangle_mesh(&collada, 0) {
//         println!("Successfully converted DAE to triangle mesh");
//     }
// }

/// Converts a Collada DAE mesh to a Bevy triangle mesh
///
/// # Arguments
///
/// * `scene` - The Scene object loaded with mesh_loader's collada::from_str
/// * `mesh_index` - The index of the mesh to convert from the Scene
///
/// # Returns
///
/// * `Option<Mesh>` - A Bevy mesh if conversion was successful, None otherwise
pub fn dae_to_triangle_mesh(scene: &Scene, mesh_index: usize) -> Option<Mesh> {
    if mesh_index >= scene.meshes.len() {
        return None;
    }

    let mesh_loader_mesh = &scene.meshes[mesh_index];

    // In Bevy 0.16, we need to add RenderAssetUsages as the second parameter
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

    // Process vertices (positions)
    if !mesh_loader_mesh.vertices.is_empty() {
        let positions: Vec<[f32; 3]> = mesh_loader_mesh.vertices
            .iter()
            .map(|v| [v[0], v[1], v[2]])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    } else {
        return None; // Vertices are required
    }

    // Process normals if available
    if !mesh_loader_mesh.normals.is_empty() {
        let normals: Vec<[f32; 3]> = mesh_loader_mesh.normals
            .iter()
            .map(|n| [n[0], n[1], n[2]])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }

    // Process texcoords (UVs) if available - mesh_loader stores UVs in an array
    // Check if we have texture coordinates in the first texture coordinate set (index 0)
    if mesh_loader_mesh.texcoords.len() > 0 && !mesh_loader_mesh.texcoords[0].is_empty() {
        let uvs: Vec<[f32; 2]> = mesh_loader_mesh.texcoords[0]
            .iter()
            .map(|uv| [uv[0], uv[1]])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    // Process faces (indices)
    if !mesh_loader_mesh.faces.is_empty() {
        // Flatten the faces into a list of indices
        let indices: Vec<u32> = mesh_loader_mesh.faces
            .iter()
            .flat_map(|face| face.iter().map(|&idx| idx as u32))
            .collect();
        mesh.insert_indices(Indices::U32(indices));
    } else {
        // If no faces, assume vertices are already arranged as triangles
        let vertex_count = mesh_loader_mesh.vertices.len();
        let indices: Vec<u32> = (0..vertex_count as u32).collect();
        mesh.insert_indices(Indices::U32(indices));
    }

    // Calculate tangents if we have positions, normals, and UVs
    if
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some() &&
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some() &&
        mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some()
    {
        mesh.generate_tangents().ok();
    }

    Some(mesh)
}
