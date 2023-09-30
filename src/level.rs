use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy_common_assets::ron::RonAssetPlugin;

use crate::AppState;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<LevelFile>::new(&["ron"]))
            .add_systems(PreUpdate, load_level.run_if(in_state(AppState::Loading)));
    }
}

#[derive(serde::Deserialize, TypeUuid, TypePath)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct LevelFile {
    pub layout: Vec<String>,
}

#[derive(Resource)]
pub struct LoadLevel(pub Handle<LevelFile>);

fn load_level(
    level: Res<LoadLevel>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    assets_level: ResMut<Assets<LevelFile>>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(level) = assets_level.get(&level.0) {
        let floor = asset_server.load("floor.glb#Scene0");
        let loading_floor = asset_server.load("loading_floor.glb#Scene0");
        let wall = asset_server.load("wall.glb#Scene0");

        for (i, row) in level.layout.iter().enumerate() {
            for (j, tile) in row.chars().enumerate() {
                match tile {
                    ' ' => cmds.spawn(SceneBundle {
                        scene: floor.clone(),
                        transform: Transform::from_xyz(i as f32, 0.0, j as f32),
                        ..Default::default()
                    }),
                    'L' => cmds.spawn(SceneBundle {
                        scene: loading_floor.clone(),
                        transform: Transform::from_xyz(i as f32, 0.0, j as f32),
                        ..Default::default()
                    }),
                    'I' => cmds.spawn(SceneBundle {
                        scene: wall.clone(),
                        transform: Transform::from_xyz(i as f32, 0.0, j as f32),
                        ..Default::default()
                    }),
                    'O' => cmds.spawn(SceneBundle {
                        scene: wall.clone(),
                        transform: Transform::from_xyz(i as f32, 0.0, j as f32),
                        ..Default::default()
                    }),
                    '#' => cmds.spawn(SceneBundle {
                        scene: wall.clone(),
                        transform: Transform::from_xyz(i as f32, 0.0, j as f32),
                        ..Default::default()
                    }),
                    _ => panic!("Unknown tile: '{}'", tile),
                };
            }
        }
        state.set(AppState::Level);
    }
}
