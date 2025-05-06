# bevy_dae

A Bevy plugin for loading and rendering DAE (Collada) files in the Bevy game engine.

[![Crates.io](https://img.shields.io/crates/v/bevy_dae)](https://crates.io/crates/bevy_dae)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

## Features

- Load and render DAE (Collada) files in Bevy
- Convert Collada meshes to Bevy meshes
- Support for meshes, materials, and textures
- Compatible with Bevy 0.16.0

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bevy_dae = "0.1.0"
```

## Usage

```rust
use bevy::prelude::*;
use bevy_dae::ColladaPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ColladaPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // Load and spawn a COLLADA model
    commands.spawn((
        Mesh3d(asset_server.load("models/your_model.dae")),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.5,
                metallic: 0.2,
                ..default()
            })
        ),
        Transform::from_scale(Vec3::splat(0.5)),
    ));

    // Add a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 5.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        ..default()
    });

    // Add some light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
```

## Utility Functions

The library also provides utility functions for working with DAE files:

```rust
use bevy_dae::dae_to_triangle_mesh;
use mesh_loader::collada::from_str;

// Load a Collada file
let collada_str = include_str!("models/your_model.dae");
let collada = from_str(collada_str).unwrap();

// Convert a DAE mesh to a Bevy triangle mesh
if let Some(triangle_mesh) = dae_to_triangle_mesh(&collada, 0) {
    // Use the converted mesh...
}
```

## Examples

Check out the examples folder for working examples:

- `joint.rs` - Demonstrates loading and rendering a jointed model

Run an example with:

```bash
cargo run --example joint
```

## License

Licensed under MIT license ([LICENSE](LICENSE)).

## Contributing

Contributions are welcome! Feel free to submit a Pull Request.

## Acknowledgments

- Built on top of the `mesh-loader` crate for Collada file parsing
- Special thanks to the Bevy community for their support and guidance
