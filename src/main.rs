use std::io::Cursor;
use thiserror::Error;
use bevy::{
    asset::{ io::Reader, AssetLoader, LoadContext },
    prelude::*,
    render::{
        mesh::{ Indices, Mesh, VertexAttributeValues },
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
use collada::document::ColladaDocument;

pub struct ColladaPlugin;
impl Plugin for ColladaPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ColladaLoader>();
    }
}

#[derive(Default)]
struct ColladaLoader;

impl AssetLoader for ColladaLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = ColladaError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        #[allow(unused_variables)] load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let collada_doc = ColladaDocument::from_xml(&String::from_utf8_lossy(&bytes))?;

        #[cfg(feature = "wireframe")]
        load_context.labeled_asset_scope("wireframe".to_string(), |_load_context| {
            collada_to_wireframe_mesh(&collada_doc)
        });

        Ok(collada_to_triangle_mesh(&collada_doc)?)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dae"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
enum ColladaError {
    #[error("Failed to load COLLADA file")] Io(#[from] std::io::Error),

    #[error("Failed to parse COLLADA XML")] Parse(#[from] collada::Error),

    #[error("Failed to process COLLADA geometry: {0}")] Geometry(String),
}

fn collada_to_triangle_mesh(collada_doc: &ColladaDocument) -> Result<Mesh, ColladaError> {
    // Get the first visual scene
    let scene = collada_doc
        .get_visual_scene()
        .ok_or_else(|| ColladaError::Geometry("No visual scene found".to_string()))?;

    // Find the first geometry node with mesh data
    let mut found_geometry = None;
    for node in &scene.nodes {
        if let Some(instance_geometry) = node.instance_geometry.as_ref() {
            found_geometry = collada_doc.get_geometry(&instance_geometry.url);
            if found_geometry.is_some() {
                break;
            }
        }
    }

    let geometry = found_geometry.ok_or_else(||
        ColladaError::Geometry("No geometry found".to_string())
    )?;

    let mesh = &geometry.mesh;

    // Get position source
    let position_source = mesh.sources
        .iter()
        .find(|s| (s.id.contains("position") || s.id.contains("Position")))
        .ok_or_else(|| ColladaError::Geometry("No position source found".to_string()))?;

    let position_stride = position_source.technique_common.accessor.stride;
    let positions_raw = &position_source.float_array.0;

    // Get normal source if available
    let normal_source = mesh.sources
        .iter()
        .find(|s| (s.id.contains("normal") || s.id.contains("Normal")));

    // Get the triangles
    let triangles = mesh.triangles
        .get(0)
        .ok_or_else(|| ColladaError::Geometry("No triangles found".to_string()))?;

    // Create a new Bevy mesh
    let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    // Extract vertex positions
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Find input semantic indices
    let mut position_offset = None;
    let mut normal_offset = None;

    let vertex_input = triangles.inputs.iter().find(|input| input.semantic == "VERTEX");

    if let Some(vertex_input) = vertex_input {
        for input in &triangles.inputs {
            if input.semantic == "VERTEX" {
                position_offset = Some(input.offset as usize);
            } else if input.semantic == "NORMAL" {
                normal_offset = Some(input.offset as usize);
            }
        }
    } else {
        // Direct position and normal inputs
        for input in &triangles.inputs {
            if input.semantic == "POSITION" {
                position_offset = Some(input.offset as usize);
            } else if input.semantic == "NORMAL" {
                normal_offset = Some(input.offset as usize);
            }
        }
    }

    let position_offset = position_offset.ok_or_else(||
        ColladaError::Geometry("No position input found".to_string())
    )?;

    let stride = triangles.inputs
        .iter()
        .map(|input| input.offset)
        .max()
        .map(|max| max + 1)
        .unwrap_or(1) as usize;

    // Process vertices and indices
    let mut vertex_map = std::collections::HashMap::new();

    for i in (0..triangles.p.len()).step_by(stride) {
        let p_indices = &triangles.p[i..i + stride];

        // Extract position data
        let pos_idx = (p_indices[position_offset] as usize) * position_stride;
        let position = [
            positions_raw[pos_idx] as f32,
            positions_raw[pos_idx + 1] as f32,
            positions_raw[pos_idx + 2] as f32,
        ];

        // Extract normal data if available
        let normal = if let Some(normal_offset) = normal_offset {
            if let Some(normal_source) = normal_source {
                let normal_stride = normal_source.technique_common.accessor.stride;
                let normals_raw = &normal_source.float_array.0;
                let norm_idx = (p_indices[normal_offset] as usize) * normal_stride;

                [
                    normals_raw[norm_idx] as f32,
                    normals_raw[norm_idx + 1] as f32,
                    normals_raw[norm_idx + 2] as f32,
                ]
            } else {
                [0.0, 1.0, 0.0] // Default normal
            }
        } else {
            [0.0, 1.0, 0.0] // Default normal
        };

        // Store vertex data and get index
        let vertex_key = (
            (position[0] * 1000.0).round() as i32,
            (position[1] * 1000.0).round() as i32,
            (position[2] * 1000.0).round() as i32,
        );

        let index = if let Some(&idx) = vertex_map.get(&vertex_key) {
            idx
        } else {
            let idx = positions.len();
            positions.push(position);
            normals.push(normal);
            vertex_map.insert(vertex_key, idx);
            idx
        };

        indices.push(index as u32);
    }

    // Generate UVs (default or from COLLADA data if available)
    let uvs = vec![[0.0, 0.0]; positions.len()];

    // Insert mesh data
    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions)
    );

    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));

    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    bevy_mesh.insert_indices(Indices::U32(indices));

    Ok(bevy_mesh)
}

#[cfg(feature = "wireframe")]
fn collada_to_wireframe_mesh(collada_doc: &ColladaDocument) -> Result<Mesh, ColladaError> {
    let scene = collada_doc
        .get_visual_scene()
        .ok_or_else(|| ColladaError::Geometry("No visual scene found".to_string()))?;

    // Find the first geometry node with mesh data
    let mut found_geometry = None;
    for node in &scene.nodes {
        if let Some(instance_geometry) = node.instance_geometry.as_ref() {
            found_geometry = collada_doc.get_geometry(&instance_geometry.url);
            if found_geometry.is_some() {
                break;
            }
        }
    }

    let geometry = found_geometry.ok_or_else(||
        ColladaError::Geometry("No geometry found".to_string())
    )?;

    let mesh = &geometry.mesh;

    // Get position source
    let position_source = mesh.sources
        .iter()
        .find(|s| (s.id.contains("position") || s.id.contains("Position")))
        .ok_or_else(|| ColladaError::Geometry("No position source found".to_string()))?;

    let position_stride = position_source.technique_common.accessor.stride;
    let positions_raw = &position_source.float_array.0;

    // Create a new Bevy mesh
    let mut bevy_mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());

    // Extract unique vertices
    let mut positions = Vec::new();
    let mut vertex_indices = std::collections::HashMap::new();

    for i in 0..positions_raw.len() / position_stride {
        let pos_idx = i * position_stride;
        let position = [
            positions_raw[pos_idx] as f32,
            positions_raw[pos_idx + 1] as f32,
            positions_raw[pos_idx + 2] as f32,
        ];

        let vertex_key = (
            (position[0] * 1000.0).round() as i32,
            (position[1] * 1000.0).round() as i32,
            (position[2] * 1000.0).round() as i32,
        );

        if !vertex_indices.contains_key(&vertex_key) {
            vertex_indices.insert(vertex_key, positions.len());
            positions.push(position);
        }
    }

    // Get triangles and create line indices
    let triangles = mesh.triangles
        .get(0)
        .ok_or_else(|| ColladaError::Geometry("No triangles found".to_string()))?;

    let position_offset = triangles.inputs
        .iter()
        .find(|input| (input.semantic == "VERTEX" || input.semantic == "POSITION"))
        .map(|input| input.offset as usize)
        .ok_or_else(|| ColladaError::Geometry("No position input found".to_string()))?;

    let stride = triangles.inputs
        .iter()
        .map(|input| input.offset)
        .max()
        .map(|max| max + 1)
        .unwrap_or(1) as usize;

    let mut line_indices = Vec::new();

    for face in (0..triangles.p.len()).step_by(stride * 3) {
        let p_indices = &triangles.p[face..face + stride * 3];

        // Extract vertex indices for this face
        let v0_key = {
            let pos_idx = (p_indices[position_offset] as usize) * position_stride;
            let pos = [
                positions_raw[pos_idx] as f32,
                positions_raw[pos_idx + 1] as f32,
                positions_raw[pos_idx + 2] as f32,
            ];
            (
                (pos[0] * 1000.0).round() as i32,
                (pos[1] * 1000.0).round() as i32,
                (pos[2] * 1000.0).round() as i32,
            )
        };

        let v1_key = {
            let pos_idx = (p_indices[position_offset + stride] as usize) * position_stride;
            let pos = [
                positions_raw[pos_idx] as f32,
                positions_raw[pos_idx + 1] as f32,
                positions_raw[pos_idx + 2] as f32,
            ];
            (
                (pos[0] * 1000.0).round() as i32,
                (pos[1] * 1000.0).round() as i32,
                (pos[2] * 1000.0).round() as i32,
            )
        };

        let v2_key = {
            let pos_idx = (p_indices[position_offset + stride * 2] as usize) * position_stride;
            let pos = [
                positions_raw[pos_idx] as f32,
                positions_raw[pos_idx + 1] as f32,
                positions_raw[pos_idx + 2] as f32,
            ];
            (
                (pos[0] * 1000.0).round() as i32,
                (pos[1] * 1000.0).round() as i32,
                (pos[2] * 1000.0).round() as i32,
            )
        };

        // Add line segments for each edge of the triangle
        if
            let (Some(&v0), Some(&v1), Some(&v2)) = (
                vertex_indices.get(&v0_key),
                vertex_indices.get(&v1_key),
                vertex_indices.get(&v2_key),
            )
        {
            line_indices.push(v0 as u32);
            line_indices.push(v1 as u32);

            line_indices.push(v1 as u32);
            line_indices.push(v2 as u32);

            line_indices.push(v2 as u32);
            line_indices.push(v0 as u32);
        }
    }

    // Generate default normals and UVs
    let normals = vec![[1.0, 0.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    // Insert mesh data
    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions)
    );

    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));

    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    bevy_mesh.insert_indices(Indices::U32(line_indices));

    Ok(bevy_mesh)
}
