#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::*;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, route_cubes)
            .add_systems(PreUpdate, (process_cubes, cube_spawner));
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Cube {
    // pub state: u8,
}

#[derive(Component, Default)]
pub struct CubeProcessor(usize);

#[derive(Component)]
pub struct CubeSpawner {
    pos: Vec3,
    delay: f32,
    next: f32,
}

impl CubeSpawner {
    pub fn new(pos: Vec3, delay: f32) -> Self {
        Self {
            pos,
            delay,
            next: 0.0,
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
                        transform.with_translation(router.0[1]),
                        EaseMethod::Linear,
                        EasingType::Once {
                            duration: Duration::from_millis(750),
                        },
                    );
                    for p in router.0.iter().skip(2) {
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
        (Entity, &Parent),
        (
            With<Cube>,
            Without<EasingComponent<Transform>>,
            Without<EasingChainComponent<Transform>>,
        ),
    >,
    mut processors: Query<&mut CubeProcessor>,
    mut cmds: Commands,
) {
    for (entity, parent) in query.iter() {
        if let Ok(mut proc) = processors.get_mut(parent.get()) {
            proc.0 += 1;
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
                        scene: asset_server.load("models/cube.glb#Scene0"),
                        transform: Transform::from_translation(spawner.pos),
                        ..Default::default()
                    },
                    Cube {},
                ));
            });
            spawner.next = time.elapsed_seconds() + spawner.delay;
        }
    }
}
