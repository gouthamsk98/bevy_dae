use bevy::prelude::*;
use bevy_dae::ColladaPlugin;
use core::f32::consts::PI;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ColladaPlugin)
        .insert_resource(SpinTimer(Timer::from_seconds(1.0 / 60.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, spin_model)
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
                base_color: Color::rgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.5,
                metallic: 0.2,
                ..default()
            })
        ),
        Transform::from_scale(Vec3::splat(0.5)),
        // PbrBundle {
        //     mesh: asset_server.load("models/joint.dae"),
        //     material: materials.add(StandardMaterial {
        //         base_color: Color::rgb(0.8, 0.7, 0.6),
        //         perceptual_roughness: 0.5,
        //         metallic: 0.2,
        //         ..default()
        //     }),
        //     transform: Transform::from_scale(Vec3::splat(0.5)),
        //     ..default()
        // },
        RotatingModel {
            angle: 0.0,
            axis: Vec3::Y,
            speed: 0.5,
        },
    ));

    // Add lighting
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 8.0, 5.0),
        ..default()
    });

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.3, 0.3, 0.3),
        brightness: 0.3,
    });

    // Add camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 4.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }),
        camera: Camera {
            hdr: true,
            ..default()
        },
        ..default()
    });
}

fn spin_model(
    time: Res<Time>,
    mut timer: ResMut<SpinTimer>,
    mut query: Query<(&mut RotatingModel, &mut Transform)>
) {
    if timer.0.tick(Duration::from_secs_f32(time.delta_secs())).just_finished() {
        for (mut model, mut transform) in query.iter_mut() {
            model.angle += (model.speed * PI) / 180.0;

            // Create rotation quaternion around the model's rotation axis
            let rotation = Quat::from_axis_angle(model.axis, model.angle);

            // Update transform rotation
            transform.rotation = rotation;
        }
    }
}
