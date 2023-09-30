use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::*;
use bevy_mod_picking::prelude::*;

pub struct CameraMovePlugin;

impl Plugin for CameraMovePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            // .add_systems(Update, animate_camera_direction)
            .add_systems(PreUpdate, camera_rot)
            .add_systems(PostUpdate, (camera_move, camera_unobstruct));
    }
}

#[derive(Component)]
pub struct Unobstruct {
    pub nw: f32,
    pub ne: f32,
    pub se: f32,
    pub sw: f32,
}

#[derive(Component)]
struct CameraDolly;

#[derive(Component)]
struct CameraArm;

fn setup(mut commands: Commands) {
    let rot = Quat::from_rotation_y(PI * 0.25);
    commands
        .spawn((
            CameraDolly,
            TransformBundle::from_transform(Transform::from_rotation(rot)),
        ))
        .with_children(|p| {
            p.spawn((CameraArm, TransformBundle { ..default() }))
                .with_children(|p| {
                    p.spawn((
                        Camera3dBundle {
                            transform: Transform::from_xyz(10.0, 12.0, 0.0)
                                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                            ..default()
                        },
                        RaycastPickCamera::default(),
                    ));
                });
        });
}

fn camera_rot(
    query: Query<(&Transform, Entity), With<CameraArm>>,
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    for (camera, entity) in query.iter() {
        if keys.just_pressed(KeyCode::E) {
            let mut rot = camera.rotation.to_euler(EulerRot::YXZ).0;
            rot = ((rot / (PI * 0.5) + 0.1).floor() + 1.0) * (PI * 0.5);
            commands.entity(entity).insert(camera.ease_to(
                camera.with_rotation(Quat::from_rotation_y(rot)),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: Duration::from_millis(750),
                },
            ));
        }
        if keys.just_pressed(KeyCode::Q) {
            let mut rot = camera.rotation.to_euler(EulerRot::YXZ).0;
            rot = ((rot / (PI * 0.5) - 0.1).ceil() - 1.0) * (PI * 0.5);
            commands.entity(entity).insert(camera.ease_to(
                camera.with_rotation(Quat::from_rotation_y(rot)),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: Duration::from_millis(750),
                },
            ));
        }
    }
}

fn camera_move(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraDolly>>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
    keys: Res<Input<KeyCode>>,
) {
    let camera = camera.single();
    for mut transform in query.iter_mut() {
        let mut input = Vec3::default();
        if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
            input.z -= 1.0;
        }
        if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
            input.z += 1.0;
        }
        if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
            input.x -= 1.0;
        }
        if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
            input.x += 1.0;
        }
        if input.length_squared() < 0.001 {
            return;
        }
        let rot = camera
            .to_scale_rotation_translation()
            .1
            .to_euler(EulerRot::YXZ)
            .0;
        transform.translation +=
            Quat::from_rotation_y(rot) * (input.normalize() * time.delta_seconds() * 5.0);
    }
}

#[allow(dead_code)]
fn animate_camera_direction(time: Res<Time>, mut query: Query<&mut Transform, With<CameraArm>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * PI / 10.0);
    }
}

fn camera_unobstruct(
    mut query: Query<(&mut Transform, &Unobstruct)>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
) {
    let camera = camera.single();
    let rot = camera
        .to_scale_rotation_translation()
        .1
        .to_euler(EulerRot::YXZ)
        .0;
    let rot = (-rot + PI * 2.5) % (2.0 * PI);
    if rot < PI * 0.25 {
        let lerp = (rot + PI * 0.25) / (PI * 0.5);
        for (mut t, u) in query.iter_mut() {
            t.translation.y = lerp * u.nw + (1.0 - lerp) * u.ne;
        }
    } else if rot < PI * 0.75 {
        let lerp = (rot - (PI * 0.25)) / (PI * 0.5);
        for (mut t, u) in query.iter_mut() {
            t.translation.y = lerp * u.sw + (1.0 - lerp) * u.nw;
        }
    } else if rot < PI * 1.25 {
        let lerp = (rot - (PI * 0.75)) / (PI * 0.5);
        for (mut t, u) in query.iter_mut() {
            t.translation.y = lerp * u.se + (1.0 - lerp) * u.sw;
        }
    } else if rot < PI * 1.75 {
        let lerp = (rot - (PI * 1.25)) / (PI * 0.5);
        for (mut t, u) in query.iter_mut() {
            t.translation.y = lerp * u.ne + (1.0 - lerp) * u.se;
        }
    } else {
        let lerp = (rot - (PI * 1.75)) / (PI * 0.5);
        for (mut t, u) in query.iter_mut() {
            t.translation.y = lerp * u.nw + (1.0 - lerp) * u.ne;
        }
    }
}
