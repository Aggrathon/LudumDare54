use std::f32::consts::*;

use bevy::prelude::*;
use bevy_easings::EasingsPlugin;

use camera::CameraMovePlugin;
use cubes::CubePlugin;
use level::LevelPlugin;
use load::{LoadLevel, LoadPlugin};

mod camera;
mod cubes;
mod level;
mod load;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 4.0f32,
        })
        .add_state::<AppState>()
        .add_plugins((
            DefaultPlugins,
            LoadPlugin,
            CameraMovePlugin,
            EasingsPlugin,
            LevelPlugin,
            CubePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        // .add_systems(Update, make_pickable)
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
