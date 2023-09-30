use bevy::prelude::*;
use level::{LevelPlugin, LoadLevel};
use std::f32::consts::*;

mod level;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 8.0f32,
        })
        .add_state::<AppState>()
        .add_plugins((DefaultPlugins, LevelPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, animate_camera_direction)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Loading,
    Level,
    Unloading,
}

fn setup(
    mut state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::new(1.0, 0.0, 1.0), Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
    commands.insert_resource(LoadLevel(asset_server.load("levels/test.ron")));
    state.set(AppState::Loading);
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        *transform = Transform::from_xyz(0.0, 3.0, 0.0).looking_at(
            Vec3::new(
                f32::sin(time.elapsed_seconds() * PI / 20.0),
                0.0,
                f32::cos(time.elapsed_seconds() * PI / 20.0),
            ),
            Vec3::Y,
        );
    }
}

fn animate_camera_direction(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut query {
        *transform = Transform::from_xyz(
            f32::sin(time.elapsed_seconds() * PI / 10.0) * 10.0,
            10.0,
            f32::cos(time.elapsed_seconds() * PI / 10.0) * 10.0,
        )
        .looking_at(Vec3::ZERO, Vec3::Y);
    }
}