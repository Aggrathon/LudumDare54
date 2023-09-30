use std::f32::consts::PI;

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

#[derive(Default, Clone, Copy)]
pub enum Tile {
    #[default]
    Empty,
    Wall(f32, f32, f32, f32),
    Floor,
    Loadingbay,
    Input(f32),
    Output(f32),
    Door(f32, f32, f32, f32, f32),
}

impl Tile {
    fn is_floor(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Loadingbay)
    }

    fn obstructable(&self) -> bool {
        !matches!(
            self,
            Tile::Empty | Tile::Door(_, _, _, _, _) | Tile::Wall(_, _, _, _)
        )
    }

    fn is_loadingbay(&self) -> bool {
        matches!(self, Tile::Loadingbay)
    }
}

#[derive(Resource)]
pub struct LoadLevel(pub Handle<LevelFile>);

fn load_level(
    level: Res<LoadLevel>,
    cmds: Commands,
    asset_server: Res<AssetServer>,
    assets_level: ResMut<Assets<LevelFile>>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(level) = assets_level.get(&level.0) {
        let mut level = level_parse(&level.layout);
        level_surround(&mut level);
        level_spawn(level, cmds, asset_server);
        state.set(AppState::Level);
    }
}

fn level_parse(layout: &[String]) -> Vec<Vec<Tile>> {
    layout
        .iter()
        .map(|row| {
            row.chars()
                .map(|c| match c {
                    ' ' => Tile::Floor,
                    'E' => Tile::Empty,
                    'L' => Tile::Loadingbay,
                    'I' => Tile::Input(0.0),
                    'O' => Tile::Output(0.0),
                    '#' => Tile::Wall(1.0, 1.0, 1.0, 1.0),
                    _ => panic!("Unknown tile: '{}'", c),
                })
                .collect()
        })
        .collect()
}

fn level_surround(layout: &mut Vec<Vec<Tile>>) {
    let get = |layout: &Vec<Vec<Tile>>, i: usize, di: isize, j: usize, dj: isize| {
        let i = di + i as isize;
        let j = dj + j as isize;
        if i < 0 || j < 0 {
            return Tile::Empty;
        }
        layout
            .get(i as usize)
            .map_or(Tile::Empty, |row: &Vec<Tile>| {
                row.get(j as usize).cloned().unwrap_or_default()
            })
    };
    for i in 0..layout.len() {
        for j in 0..layout[i].len() {
            match &layout[i][j] {
                Tile::Wall(_, _, _, _) => {
                    let nw = if get(layout, i, -1, j, -1).obstructable() {
                        0.0
                    } else if get(layout, i, -2, j, -2).obstructable() {
                        0.5
                    } else {
                        1.0
                    };
                    let ne = if get(layout, i, -1, j, 1).obstructable() {
                        0.0
                    } else if get(layout, i, -2, j, 2).obstructable() {
                        0.5
                    } else {
                        1.0
                    };
                    let se = if get(layout, i, 1, j, 1).obstructable() {
                        0.0
                    } else if get(layout, i, 2, j, 2).obstructable() {
                        0.5
                    } else {
                        1.0
                    };
                    let sw = if get(layout, i, 1, j, -1).obstructable() {
                        0.0
                    } else if get(layout, i, 2, j, -2).obstructable() {
                        0.5
                    } else {
                        1.0
                    };
                    if get(layout, i, 0, j, 1).is_loadingbay() {
                        layout[i][j] = Tile::Door(PI * 0.5, nw, ne, se, sw);
                    } else if get(layout, i, 1, j, 0).is_loadingbay() {
                        layout[i][j] = Tile::Door(0.0, nw, ne, se, sw);
                    } else if get(layout, i, 0, j, -1).is_loadingbay() {
                        layout[i][j] = Tile::Door(-PI * 0.5, nw, ne, se, sw);
                    } else if get(layout, i, -1, j, 0).is_loadingbay() {
                        layout[i][j] = Tile::Door(PI, nw, ne, se, sw);
                    } else {
                        layout[i][j] = Tile::Wall(nw, ne, se, sw);
                    }
                }
                Tile::Input(_) => {
                    if get(layout, i, 0, j, 1).is_floor() {
                        layout[i][j] = Tile::Input(-PI * 0.5)
                    } else if get(layout, i, -1, j, 0).is_floor() {
                        layout[i][j] = Tile::Input(PI)
                    } else if get(layout, i, 0, j, -1).is_floor() {
                        layout[i][j] = Tile::Input(PI * 0.5)
                    }
                }
                Tile::Output(_) => {
                    if get(layout, i, 0, j, 1).is_floor() {
                        layout[i][j] = Tile::Output(-PI * 0.5)
                    } else if get(layout, i, -1, j, 0).is_floor() {
                        layout[i][j] = Tile::Output(PI)
                    } else if get(layout, i, 0, j, -1).is_floor() {
                        layout[i][j] = Tile::Output(PI * 0.5)
                    }
                }
                _ => {}
            }
        }
    }
}

fn level_spawn(layout: Vec<Vec<Tile>>, mut cmds: Commands, asset_server: Res<AssetServer>) {
    let floor = asset_server.load("models/floor.glb#Scene0");
    let loadingbay = asset_server.load("models/loadingbay.glb#Scene0");
    let wall = asset_server.load("models/wall.glb#Scene0");
    let door = asset_server.load("models/door.glb#Scene0");
    let input = asset_server.load("models/input.glb#Scene0");
    let output = asset_server.load("models/output.glb#Scene0");

    let offset_x = -(layout.len() as f32 * 0.5);
    let offset_y = -(layout[0].len() as f32 * 0.5);

    for (i, row) in layout.into_iter().enumerate() {
        for (j, tile) in row.into_iter().enumerate() {
            let x = offset_x + i as f32;
            let y = offset_y + j as f32;
            match tile {
                Tile::Empty => {}
                Tile::Wall(_, _, _, _) => {
                    cmds.spawn(SceneBundle {
                        scene: wall.clone(),
                        transform: Transform::from_xyz(x, 0.0, y),
                        ..Default::default()
                    });
                }
                Tile::Floor => {
                    cmds.spawn(SceneBundle {
                        scene: floor.clone(),
                        transform: Transform::from_xyz(x, 0.0, y),
                        ..Default::default()
                    });
                }
                Tile::Loadingbay => {
                    cmds.spawn(SceneBundle {
                        scene: loadingbay.clone(),
                        transform: Transform::from_xyz(x, 0.0, y),
                        ..Default::default()
                    });
                }
                Tile::Input(rot) => {
                    cmds.spawn(SceneBundle {
                        scene: input.clone(),
                        transform: Transform::from_xyz(x, 0.0, y)
                            .with_rotation(Quat::from_rotation_y(rot)),
                        ..Default::default()
                    });
                }
                Tile::Output(rot) => {
                    cmds.spawn(SceneBundle {
                        scene: output.clone(),
                        transform: Transform::from_xyz(x, 0.0, y)
                            .with_rotation(Quat::from_rotation_y(rot)),
                        ..Default::default()
                    });
                }
                Tile::Door(rot, _, _, _, _) => {
                    cmds.spawn(SceneBundle {
                        scene: door.clone(),
                        transform: Transform::from_xyz(x, 0.0, y)
                            .with_rotation(Quat::from_rotation_y(rot)),
                        ..Default::default()
                    });
                }
            };
        }
    }
}
