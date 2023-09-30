use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::camera::Unobstruct;
use crate::level::Level;
use crate::AppState;

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<LevelFile>::new(&["ron"]))
            .add_systems(PreUpdate, load_level.run_if(in_state(AppState::Loading)));
    }
}

#[derive(Resource)]
pub struct LoadLevel(pub Handle<LevelFile>);

#[derive(serde::Deserialize, TypeUuid, TypePath)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct LevelFile {
    pub layout: Vec<String>,
    pub objects: Vec<Object>,
}

#[derive(Default, Clone, Copy)]
pub enum Tile {
    #[default]
    Empty,
    Wall(f32, f32, f32, f32),
    Floor(Object),
    Loadingbay,
    Input(f32),
    Output(f32),
    Door(f32, f32, f32, f32, f32),
}

#[derive(Deserialize, Default, Clone, Copy)]
pub enum Object {
    #[default]
    Empty,
    Belt,
}

impl Tile {
    fn is_floor(&self) -> bool {
        matches!(self, Tile::Floor(_) | Tile::Loadingbay)
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

fn load_level(
    level: Res<LoadLevel>,
    cmds: Commands,
    asset_server: Res<AssetServer>,
    assets_level: ResMut<Assets<LevelFile>>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(level) = assets_level.get(&level.0) {
        let mut level = level_parse(level);
        level_surround(&mut level);
        level_spawn(level, cmds, asset_server);
        state.set(AppState::Level);
    }
}

fn level_parse(level: &LevelFile) -> Vec<Vec<Tile>> {
    level
        .layout
        .iter()
        .map(|row| {
            row.chars()
                .map(|c| match c {
                    ' ' => Tile::Floor(Object::Empty),
                    'E' => Tile::Empty,
                    'L' => Tile::Loadingbay,
                    'I' => Tile::Input(0.0),
                    'O' => Tile::Output(0.0),
                    '#' => Tile::Wall(0.0, 0.0, 0.0, 0.0),
                    '0' => Tile::Floor(level.objects[0]),
                    '1' => Tile::Floor(level.objects[1]),
                    '2' => Tile::Floor(level.objects[2]),
                    '3' => Tile::Floor(level.objects[3]),
                    '4' => Tile::Floor(level.objects[4]),
                    '5' => Tile::Floor(level.objects[5]),
                    '6' => Tile::Floor(level.objects[6]),
                    '7' => Tile::Floor(level.objects[7]),
                    '8' => Tile::Floor(level.objects[8]),
                    '9' => Tile::Floor(level.objects[9]),
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
                        -2.0
                    } else if get(layout, i, -2, j, -2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let ne = if get(layout, i, -1, j, 1).obstructable() {
                        -2.0
                    } else if get(layout, i, -2, j, 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let se = if get(layout, i, 1, j, 1).obstructable() {
                        -2.0
                    } else if get(layout, i, 2, j, 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let sw = if get(layout, i, 1, j, -1).obstructable() {
                        -2.0
                    } else if get(layout, i, 2, j, -2).obstructable() {
                        -1.0
                    } else {
                        0.0
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

    let mut level = Level::new(layout.len(), layout[0].len());
    let (offset_x, offset_y) = level.offset();

    for (i, row) in layout.into_iter().enumerate() {
        for (j, tile) in row.into_iter().enumerate() {
            let x = offset_x + i as f32;
            let y = offset_y + j as f32;
            match tile {
                Tile::Empty => {}
                Tile::Wall(nw, ne, se, sw) => {
                    cmds.spawn((
                        SceneBundle {
                            scene: wall.clone(),
                            transform: Transform::from_xyz(x, 0.0, y),
                            ..Default::default()
                        },
                        Unobstruct { nw, ne, se, sw },
                    ));
                }
                Tile::Floor(object) => {
                    cmds.spawn(SceneBundle {
                        scene: floor.clone(),
                        transform: Transform::from_xyz(x, 0.0, y),
                        ..Default::default()
                    });
                    match object {
                        Object::Empty => *level.get_mut(i, j).unwrap() = 0,
                        Object::Belt => {
                            let index = level.next_index();
                            *level.get_mut(i, j).unwrap() = index;
                            // TODO add index to object
                            cmds.spawn(SceneBundle {
                                scene: asset_server.load("models/belt.glb#Scene0"),
                                transform: Transform::from_xyz(x, 0.0, y),
                                ..Default::default()
                            });
                        }
                    }
                }
                Tile::Loadingbay => {
                    *level.get_mut(i, j).unwrap() = 0;
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
                Tile::Door(rot, nw, ne, se, sw) => {
                    cmds.spawn((
                        SceneBundle {
                            scene: door.clone(),
                            transform: Transform::from_xyz(x, 0.0, y)
                                .with_rotation(Quat::from_rotation_y(rot)),
                            ..Default::default()
                        },
                        Unobstruct { nw, ne, se, sw },
                    ));
                }
            };
        }
    }
}