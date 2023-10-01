use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::camera::Unobstruct;
use crate::cubes::{CubeColor, CubeProcessor, CubeRouter, CubeSpawner};
use crate::level::Level;
use crate::objects::BeltBuilder;
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

#[derive(Deserialize, TypeUuid, TypePath)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct LevelFile {
    pub layout: Vec<String>,
    pub objects: Vec<Object>,
}

#[derive(Default, Clone, Debug)]
pub enum Tile {
    #[default]
    Empty,
    Wall(f32, f32, f32, f32),
    Floor(Object),
    Loadingbay,
    Input(f32, CubeColor),
    Output(f32, CubeColor),
    Door(f32, f32, f32, f32, f32),
}

#[derive(Deserialize, Default, Clone, Debug)]
pub enum Object {
    #[default]
    Empty,
    Belt(String),
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
                    'I' => Tile::Input(0.0, CubeColor::Green),
                    'O' => Tile::Output(0.0, CubeColor::Green),
                    'i' => Tile::Input(0.0, CubeColor::Purple),
                    'o' => Tile::Output(0.0, CubeColor::Purple),
                    'N' => Tile::Input(0.0, CubeColor::Yellow),
                    'U' => Tile::Output(0.0, CubeColor::Yellow),
                    'n' => Tile::Input(0.0, CubeColor::Black),
                    'u' => Tile::Output(0.0, CubeColor::Black),
                    '#' => Tile::Wall(0.0, 0.0, 0.0, 0.0),
                    '0' => Tile::Floor(level.objects[0].clone()),
                    '1' => Tile::Floor(level.objects[1].clone()),
                    '2' => Tile::Floor(level.objects[2].clone()),
                    '3' => Tile::Floor(level.objects[3].clone()),
                    '4' => Tile::Floor(level.objects[4].clone()),
                    '5' => Tile::Floor(level.objects[5].clone()),
                    '6' => Tile::Floor(level.objects[6].clone()),
                    '7' => Tile::Floor(level.objects[7].clone()),
                    '8' => Tile::Floor(level.objects[8].clone()),
                    '9' => Tile::Floor(level.objects[9].clone()),
                    _ => panic!("Unknown tile: '{}'", c),
                })
                .collect()
        })
        .collect()
}

fn level_surround(layout: &mut Vec<Vec<Tile>>) {
    let get = |layout: &Vec<Vec<Tile>>, i: isize, j: isize| {
        if i < 0 || j < 0 {
            return Tile::Empty;
        }
        layout
            .get(j as usize)
            .map_or(Tile::Empty, |row: &Vec<Tile>| {
                row.get(i as usize).cloned().unwrap_or_default()
            })
    };
    for j in 0..layout.len() {
        for i in 0..layout[j].len() {
            let x = i as isize;
            let z = j as isize;
            match &layout[j][i] {
                Tile::Wall(_, _, _, _) => {
                    let nw = if get(layout, x - 1, z - 1).obstructable() {
                        -2.0
                    } else if get(layout, x - 2, z - 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let ne = if get(layout, x - 1, z + 1).obstructable() {
                        -2.0
                    } else if get(layout, x - 2, z + 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let se = if get(layout, x + 1, z + 1).obstructable() {
                        -2.0
                    } else if get(layout, x + 2, z + 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    let sw = if get(layout, x + 1, z - 1).obstructable() {
                        -2.0
                    } else if get(layout, x + 2, z - 2).obstructable() {
                        -1.0
                    } else {
                        0.0
                    };
                    if get(layout, x, z + 1).is_loadingbay() {
                        layout[j][i] = Tile::Door(PI * 0.5, nw, ne, se, sw);
                    } else if get(layout, x + 1, z).is_loadingbay() {
                        layout[j][i] = Tile::Door(0.0, nw, ne, se, sw);
                    } else if get(layout, x, z - 1).is_loadingbay() {
                        layout[j][i] = Tile::Door(-PI * 0.5, nw, ne, se, sw);
                    } else if get(layout, x - 1, z).is_loadingbay() {
                        layout[j][i] = Tile::Door(PI, nw, ne, se, sw);
                    } else {
                        layout[j][i] = Tile::Wall(nw, ne, se, sw);
                    }
                }
                Tile::Input(_, t) => {
                    if get(layout, x + 1, z).is_floor() {
                        layout[j][i] = Tile::Input(PI, *t)
                    } else if get(layout, x, z + 1).is_floor() {
                        layout[j][i] = Tile::Input(PI * 0.5, *t)
                    } else if get(layout, x, z - 1).is_floor() {
                        layout[j][i] = Tile::Input(-PI * 0.5, *t)
                    }
                }
                Tile::Output(_, t) => {
                    if get(layout, x + 1, z).is_floor() {
                        layout[j][i] = Tile::Output(PI * 0.5, *t)
                    } else if get(layout, x - 1, z).is_floor() {
                        layout[j][i] = Tile::Output(-PI * 0.5, *t)
                    } else if get(layout, x, z - 1).is_floor() {
                        layout[j][i] = Tile::Output(PI, *t)
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

    let mut level = Level::new(layout[0].len(), layout.len());
    let offset = level.offset();

    for (j, row) in layout.into_iter().enumerate() {
        for (i, tile) in row.into_iter().enumerate() {
            let pos = offset + Vec3::new(i as f32, 0.0, j as f32);
            match tile {
                Tile::Empty => {}
                Tile::Wall(nw, ne, se, sw) => {
                    cmds.spawn((
                        SceneBundle {
                            scene: wall.clone(),
                            transform: Transform::from_translation(pos),
                            ..Default::default()
                        },
                        Unobstruct { nw, ne, se, sw },
                    ));
                }
                Tile::Floor(object) => {
                    cmds.spawn(SceneBundle {
                        scene: floor.clone(),
                        transform: Transform::from_translation(pos),
                        ..Default::default()
                    });
                    spawn_object(object, i, j, pos, &mut level, &mut cmds, &asset_server);
                }
                Tile::Loadingbay => {
                    level.set_floor(i, j);
                    cmds.spawn(SceneBundle {
                        scene: loadingbay.clone(),
                        transform: Transform::from_translation(pos),
                        ..Default::default()
                    });
                }
                Tile::Input(rot, color) => {
                    // TODO color based on type
                    cmds.spawn((
                        SceneBundle {
                            scene: asset_server.load(color.io_path()),
                            transform: Transform::from_translation(pos)
                                .with_rotation(Quat::from_rotation_y(rot + PI * 0.5)),
                            ..Default::default()
                        },
                        CubeSpawner::new(Vec3::Y, 2.0, color),
                        CubeRouter(vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, -0.5)]),
                    ));
                }
                Tile::Output(rot, color) => {
                    cmds.spawn((
                        SceneBundle {
                            scene: asset_server.load(color.io_path()),
                            transform: Transform::from_translation(pos)
                                .with_rotation(Quat::from_rotation_y(rot)),
                            ..Default::default()
                        },
                        CubeProcessor::new(color),
                        CubeRouter(vec![Vec3::new(0.0, 1.0, 0.5), Vec3::new(0.0, 1.0, 0.0)]),
                    ));
                }
                Tile::Door(rot, nw, ne, se, sw) => {
                    cmds.spawn((
                        SceneBundle {
                            scene: door.clone(),
                            transform: Transform::from_translation(pos)
                                .with_rotation(Quat::from_rotation_y(rot)),
                            ..Default::default()
                        },
                        Unobstruct { nw, ne, se, sw },
                    ));
                }
            };
        }
    }
    cmds.insert_resource(level);
}

fn spawn_object(
    object: Object,
    i: usize,
    j: usize,
    pos: Vec3,
    level: &mut Level,
    cmds: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    match object {
        Object::Empty => level.set_floor(i, j),
        Object::Belt(path) => {
            let mut bb = BeltBuilder::new();
            for c in path.chars() {
                match c {
                    'f' | 'F' => bb = bb.forward(asset_server),
                    'l' | 'L' => bb = bb.left(asset_server),
                    'r' | 'R' => bb = bb.right(asset_server),
                    _ => panic!("Unknown direction"),
                }
            }
            bb.build((i, j).into(), pos, level, cmds);
        }
    }
}
