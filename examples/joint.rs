use bevy::prelude::*;
use bevy_dae::ColladaPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ColladaPlugin)
        .insert_resource(SpinTimer(Timer::from_seconds(1.0 / 60.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct RotatingModel {
    angle: f32,
    axis: Vec3,
    speed: f32,
}
#[derive(Resource)]
struct SpinTimer(Timer);

#[allow(deprecated)]
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // Load and spawn our COLLADA model
    commands.spawn((
        Mesh3d(asset_server.load("models/joint.dae")),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.5,
                metallic: 0.2,
                ..default()
            })
        ),
        Transform::from_scale(Vec3::splat(0.5)),
        RotatingModel {
            angle: 0.0,
            axis: Vec3::Y,
            speed: 0.5,
        },
    ));

    // Add lighting
    commands.spawn((
        Transform::from_xyz(30.0, 0.0, 20.0),
        PointLight {
            range: 40.0,
            ..Default::default()
        },
    ));

    // Add camera
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -1.0, 1.0)).looking_at(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y
        ),
        Camera3d::default(),
        Msaa::Sample4,
    ));
}
