#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::*;

use crate::levels::LevelEntity;
use crate::ui::ShowVictory;
use crate::AppState;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CubeRecieved>()
            .add_systems(PreUpdate, route_cubes)
            .add_systems(
                Update,
                (process_cubes, cube_spawner, check_connection).run_if(in_state(AppState::Level)),
            );
    }
}

#[derive(Event)]
struct CubeRecieved;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeColor {
    Green,
    Purple,
    Yellow,
    Black,
}

impl CubeColor {
    pub fn cube_path(&self) -> &str {
        match self {
            CubeColor::Green => "models/cubeG.glb#Scene0",
            CubeColor::Purple => "models/cubeP.glb#Scene0",
            CubeColor::Yellow => "models/cubeY.glb#Scene0",
            CubeColor::Black => "models/cubeB.glb#Scene0",
        }
    }

    pub fn io_path(&self) -> &str {
        match self {
            CubeColor::Green => "models/inoutG.glb#Scene0",
            CubeColor::Purple => "models/inoutP.glb#Scene0",
            CubeColor::Yellow => "models/inoutY.glb#Scene0",
            CubeColor::Black => "models/inoutB.glb#Scene0",
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Cube(CubeColor);

#[derive(Component, Debug)]
pub struct CubeProcessor {
    count: usize,
    color: CubeColor,
}

impl CubeProcessor {
    pub fn new(color: CubeColor) -> Self {
        Self { count: 0, color }
    }
}

#[derive(Component)]
pub struct CubeSpawner {
    pos: Vec3,
    delay: f32,
    next: f32,
    color: CubeColor,
}

impl CubeSpawner {
    pub fn new(pos: Vec3, delay: f32, color: CubeColor) -> Self {
        Self {
            pos,
            delay,
            next: 0.0,
            color,
        }
    }
}

#[derive(Component)]
pub struct CubeRouter(pub Vec<Vec3>);

fn route_cubes(
    query: Query<(Entity, &Transform), (With<Cube>, Without<Parent>)>,
    routers: Query<(Entity, &CubeRouter, &GlobalTransform)>,
    mut cmds: Commands,
) {
    for (entity, transform) in query.iter() {
        let mut parent = false;
        for (rent, router, rtrans) in routers.iter() {
            let pos = rtrans.transform_point(router.0[0]);
            if (transform.translation - pos).length_squared() < 0.01 {
                cmds.entity(rent).add_child(entity);
                let eases = transform.with_translation(router.0[0]).ease_to(
                    transform.with_translation(router.0[1]),
                    EaseMethod::Linear,
                    EasingType::Once {
                        duration: Duration::from_millis(750),
                    },
                );
                if router.0.len() > 2 {
                    let mut eases = eases.ease_to(
                        transform.with_translation(router.0[2]),
                        EaseMethod::Linear,
                        EasingType::Once {
                            duration: Duration::from_millis(750),
                        },
                    );
                    for p in router.0.iter().skip(3) {
                        eases = eases.ease_to(
                            transform.with_translation(*p),
                            EaseMethod::Linear,
                            EasingType::Once {
                                duration: Duration::from_millis(750),
                            },
                        );
                    }
                    cmds.entity(entity).insert(eases);
                } else {
                    cmds.entity(entity).insert(eases);
                }
                parent = true;
                break;
            }
        }
        if !parent {
            cmds.entity(entity).despawn_recursive();
        }
    }
}

fn process_cubes(
    query: Query<
        (Entity, &Parent, &Cube),
        (
            Without<EasingComponent<Transform>>,
            Without<EasingChainComponent<Transform>>,
        ),
    >,
    mut processors: Query<&mut CubeProcessor>,
    mut event: EventWriter<CubeRecieved>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
) {
    for (entity, parent, cube) in query.iter() {
        if let Ok(mut proc) = processors.get_mut(parent.get()) {
            if cube.0 == proc.color {
                proc.count += 1;
                event.send(CubeRecieved);
            } else {
                cmds.spawn(AudioBundle {
                    source: asset_server.load("sounds/boop.ogg"),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new_relative(0.5),
                        speed: fastrand::f32() * 0.3 + 0.8,
                        paused: false,
                    },
                });
            }
            cmds.entity(entity).despawn_recursive();
        } else {
            cmds.entity(entity).remove_parent_in_place();
        }
    }
}

fn cube_spawner(
    mut query: Query<(Entity, &mut CubeSpawner)>,
    time: Res<Time>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut spawner) in query.iter_mut() {
        if spawner.next < time.elapsed_seconds() {
            cmds.entity(entity).with_children(|p| {
                p.spawn((
                    SceneBundle {
                        scene: asset_server.load(spawner.color.cube_path()),
                        transform: Transform::from_translation(spawner.pos),
                        ..Default::default()
                    },
                    Cube(spawner.color),
                    LevelEntity,
                ));
            });
            spawner.next = time.elapsed_seconds() + spawner.delay;
        }
    }
}

fn check_connection(
    event: EventReader<CubeRecieved>,
    routers: Query<(Entity, &CubeRouter, &GlobalTransform)>,
    spawners: Query<&CubeSpawner>,
    processors: Query<(&CubeRouter, &GlobalTransform, &CubeProcessor)>,
    mut victory: ResMut<ShowVictory>,
) {
    if event.is_empty() {
        return;
    }
    for (_, _, processor) in processors.iter() {
        if processor.count == 0 {
            return;
        }
    }
    for (router, global, processor) in processors.iter() {
        let mut pos = global.transform_point(router.0[0]);
        'outer: loop {
            for (rent, router, rtrans) in routers.iter() {
                let pos2 = rtrans.transform_point(*router.0.last().unwrap());
                if pos.distance_squared(pos2) < 0.1 {
                    if let Ok(spawner) = spawners.get(rent) {
                        if spawner.color == processor.color {
                            break 'outer;
                        }
                    }
                    pos = rtrans.transform_point(router.0[0]);
                    continue 'outer;
                }
            }
            return;
        }
    }
    victory.show();
}
