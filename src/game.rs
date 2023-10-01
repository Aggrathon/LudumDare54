use std::f32::consts::PI;
use std::ops::{Add, AddAssign};

use bevy::math::vec4;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::AppState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, setup)
            .add_systems(
                Update,
                make_scene_draggable.run_if(in_state(AppState::Level)),
            );
    }
}

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
// }

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Default)]
pub struct Dis2 {
    x: isize,
    z: isize,
}

impl Dis2 {
    pub const fn new(x: isize, z: isize) -> Self {
        Self { x, z }
    }

    #[allow(dead_code)]
    pub const ZERO: Dis2 = Dis2::new(0, 0);
    #[allow(dead_code)]
    pub const X: Dis2 = Dis2::new(1, 0);
    #[allow(dead_code)]
    pub const NEG_X: Dis2 = Dis2::new(-1, 0);
    #[allow(dead_code)]
    pub const Z: Dis2 = Dis2::new(0, 1);
    #[allow(dead_code)]
    pub const NEG_Z: Dis2 = Dis2::new(0, -1);

    pub fn rotated(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::D0 => Self::new(self.x, self.z),
            Rotation::D90 => Self::new(-self.z, self.x),
            Rotation::D180 => Self::new(-self.x, -self.z),
            Rotation::D270 => Self::new(self.z, -self.x),
        }
    }

    pub fn distance(&self, other: Dis2) -> isize {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }
}

impl From<(isize, isize)> for Dis2 {
    fn from(value: (isize, isize)) -> Self {
        Self::new(value.0, value.1)
    }
}
impl From<(usize, usize)> for Dis2 {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0 as isize, value.1 as isize)
    }
}

impl AddAssign for Dis2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.z += rhs.z;
    }
}

impl Add for Dis2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.z + rhs.z)
    }
}

#[derive(Resource)]
pub struct Level {
    width: usize,
    height: usize,
    index: usize,
    floor: Vec<usize>,
}

impl Level {
    pub fn new(width: usize, height: usize) -> Self {
        Level {
            width,
            height,
            index: 0,
            floor: vec![usize::MAX; width * height],
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, z: usize) -> Option<&usize> {
        if x >= self.width {
            None
        } else {
            self.floor.get(z * self.width + x)
        }
    }

    pub fn getd(&self, dis: Dis2) -> Option<&usize> {
        let w = self.width as isize;
        let h = self.height as isize;
        if dis.x >= w || dis.x < 0 || dis.z > h || dis.z < 0 {
            None
        } else {
            self.floor.get((dis.z * w + dis.x) as usize)
        }
    }
    fn getd_mut(&mut self, dis: Dis2) -> Option<&mut usize> {
        let w = self.width as isize;
        let h = self.height as isize;
        if dis.x >= w || dis.x < 0 || dis.z > h || dis.z < 0 {
            None
        } else {
            self.floor.get_mut((dis.z * w + dis.x) as usize)
        }
    }

    fn get_mut(&mut self, x: usize, z: usize) -> Option<&mut usize> {
        if x >= self.width {
            None
        } else {
            self.floor.get_mut(z * self.width + x)
        }
    }

    pub fn offset(&self) -> Vec3 {
        Vec3::new(-(self.width as f32 * 0.5), 0.0, -(self.height as f32 * 0.5))
    }

    pub fn next_index(&mut self) -> usize {
        self.index += 1;
        self.index
    }

    pub fn try_place(&self, block: &Block, dis: Dis2, dir: Rotation) -> bool {
        for pos in block.iter_with(dis, dir) {
            match self.getd(pos) {
                Some(0) => {}
                Some(i) if *i == block.index => {}
                _ => return false,
            }
        }
        true
    }

    pub fn remove(&mut self, block: &Block) {
        block.iter().for_each(|d| {
            let tile = self.getd_mut(d).unwrap();
            debug_assert_eq!(*tile, block.index);
            *tile = 0;
        });
    }

    pub fn place(&mut self, block: &Block) {
        block.iter().for_each(|d| {
            let tile = self.getd_mut(d).unwrap();
            debug_assert_eq!(*tile, 0);
            *tile = block.index;
        });
    }

    pub fn place_unchecked(&mut self, block: &Block) {
        block
            .iter()
            .for_each(|d| *self.getd_mut(d).unwrap() = block.index);
    }

    pub fn set_floor(&mut self, x: usize, z: usize) {
        let tile = self.get_mut(x, z).unwrap();
        if *tile == usize::MAX {
            *tile = 0;
        }
    }

    pub fn to_discrete(&self, pos: Vec3) -> Dis2 {
        let pos = pos - self.offset();
        Dis2::new(pos.x.round() as isize, pos.z.round() as isize)
    }

    pub fn to_vec3(&self, pos: Dis2) -> Vec3 {
        self.offset() + Vec3::new(pos.x as f32, 0.0, pos.z as f32)
    }
}

impl std::fmt::Debug for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let floor = (0..self.height)
            .map(|j| {
                self.floor[(j * self.width)..((j + 1) * self.width)]
                    .iter()
                    .map(|t| {
                        let mut s = String::new();
                        if *t == 0 {
                            s += " ";
                        } else if *t == usize::MAX {
                            s += "#";
                        } else {
                            s += &t.to_string();
                        }
                        s
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        f.debug_struct("Level")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("index", &self.index)
            .field("floor", &floor)
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rotation {
    D0,
    D90,
    D180,
    D270,
}

impl Rotation {
    pub fn right(self) -> Self {
        match self {
            Rotation::D0 => Rotation::D270,
            Rotation::D90 => Rotation::D0,
            Rotation::D180 => Rotation::D90,
            Rotation::D270 => Rotation::D180,
        }
    }

    pub fn left(self) -> Self {
        match self {
            Rotation::D0 => Rotation::D90,
            Rotation::D90 => Rotation::D180,
            Rotation::D180 => Rotation::D270,
            Rotation::D270 => Rotation::D0,
        }
    }

    pub fn as_radians(&self) -> f32 {
        match self {
            Rotation::D0 => 0.0,
            Rotation::D90 => -PI * 0.5,
            Rotation::D180 => PI,
            Rotation::D270 => PI * 0.5,
        }
    }

    pub fn as_vec3(&self) -> Vec3 {
        match self {
            Rotation::D0 => Vec3::NEG_Z,
            Rotation::D90 => Vec3::NEG_X,
            Rotation::D180 => Vec3::Z,
            Rotation::D270 => Vec3::X,
        }
    }

    pub fn as_discrete(&self) -> Dis2 {
        match self {
            Rotation::D0 => Dis2::NEG_Z,
            Rotation::D90 => Dis2::NEG_X,
            Rotation::D180 => Dis2::Z,
            Rotation::D270 => Dis2::X,
        }
    }
}

impl Add for Rotation {
    type Output = Rotation;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            Rotation::D0 => self,
            Rotation::D90 => self.left(),
            Rotation::D180 => self.left().left(),
            Rotation::D270 => self.right(),
        }
    }
}

#[derive(Component)]
pub struct Block {
    pub index: usize,
    pub tiles: Vec<Dis2>,
    rotation: Rotation,
    position: Dis2,
}

impl Block {
    pub fn new(index: usize, position: Dis2) -> Self {
        Self {
            index,
            tiles: vec![Dis2::ZERO],
            rotation: Rotation::D0,
            position,
        }
    }

    #[allow(dead_code)]
    pub fn with_tile(mut self, pos: Dis2) -> Self {
        self.tiles.push(pos);
        self
    }

    #[allow(unused)]
    pub fn left(mut self) -> Self {
        self.rotation = self.rotation.left();
        self
    }

    #[allow(unused)]
    pub fn right(mut self) -> Self {
        self.rotation = self.rotation.right();
        self
    }

    pub fn translate(&mut self, position: Dis2) -> &mut Self {
        self.position = position;
        self
    }

    pub fn rotate(&mut self, rotation: Rotation) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = Dis2> + '_ {
        self.iter_with(self.position, self.rotation)
    }

    pub fn iter_with(&self, dis: Dis2, rotation: Rotation) -> impl Iterator<Item = Dis2> + '_ {
        self.tiles
            .iter()
            .map(move |dir| dis + dir.rotated(rotation))
    }
}

#[derive(Component)]
pub struct MakeSceneDraggable(pub Option<Entity>);

#[derive(Component)]
struct Draggable(Entity);

fn on_drag(
    event: Listener<Pointer<Drag>>,
    root_query: Query<&Draggable>,
    mut block_query: Query<(&mut Transform, &mut Block), With<Draggable>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut level: ResMut<Level>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    if let Ok(root) = root_query.get(event.target) {
        if let Ok((mut transform, mut block)) = block_query.get_mut(root.0) {
            let (camera, camera_transform) = camera.single();
            if let Some(ray) =
                camera.viewport_to_world(camera_transform, event.pointer_location.position)
            {
                if let Some(dist) = ray.intersect_plane(transform.translation, Vec3::Y) {
                    let pos = ray.get_point(dist);
                    let mut dis = level.to_discrete(pos);
                    let dist = dis.distance(block.position);
                    if dist > 0 {
                        if dist > 1 {
                            let mut dx = (dis.x - block.position.x).signum();
                            let mut dz = (dis.z - block.position.z).signum();
                            if dx != 0 && dz != 0 {
                                if fastrand::bool() {
                                    dx = 0;
                                } else {
                                    dz = 0;
                                }
                            }
                            dis = block.position + Dis2::new(dx, dz);
                        }
                        if level.try_place(&block, dis, block.rotation) {
                            level.remove(&block);
                            block.translate(dis);
                            level.place(&block);
                            transform.translation = level.to_vec3(dis);
                            commands.spawn(AudioBundle {
                                source: asset_server.load("sounds/click.ogg"),
                                settings: PlaybackSettings {
                                    mode: bevy::audio::PlaybackMode::Despawn,
                                    volume: bevy::audio::Volume::new_relative(0.5),
                                    speed: fastrand::f32() * 0.3 + 0.75,
                                    paused: false,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
}

fn on_click(
    event: Listener<Pointer<Click>>,
    root_query: Query<&Draggable>,
    mut block_query: Query<(&mut Transform, &mut Block), With<Draggable>>,
    mut level: ResMut<Level>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if event.button != PointerButton::Secondary {
        return;
    }
    if let Ok(root) = root_query.get(event.target) {
        if let Ok((mut transform, mut block)) = block_query.get_mut(root.0) {
            let mut dir = block.rotation;
            for _ in 0..3 {
                dir = dir.left();
                if level.try_place(&block, block.position, dir) {
                    level.remove(&block);
                    block.rotate(dir);
                    level.place(&block);
                    transform.rotation = Quat::from_rotation_y(dir.as_radians());
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sounds/clank.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new_relative(0.5),
                            speed: fastrand::f32() * 0.2 + 0.9,
                            paused: false,
                        },
                    });
                    return;
                }
            }
            commands.spawn(AudioBundle {
                source: asset_server.load("sounds/boop.ogg"),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new_relative(0.5),
                    speed: fastrand::f32() * 0.2 + 0.9,
                    paused: false,
                },
            });
        }
    }
}

fn make_scene_draggable(
    mut commands: Commands,
    mut query: Query<(Entity, &MakeSceneDraggable), With<Children>>,
    mesh_query: Query<Entity, (With<Parent>, With<Handle<Mesh>>)>,
    child_query: Query<&Children>,
) {
    for (entity, drag) in query.iter_mut() {
        match drag.0 {
            Some(e) => {
                make_pickable_recursive(&mut commands, &e, &entity, &child_query, &mesh_query)
            }
            None => {
                make_pickable_recursive(&mut commands, &entity, &entity, &child_query, &mesh_query);
                commands
                    .entity(entity)
                    .remove::<MakeSceneDraggable>()
                    .insert((
                        On::<Pointer<Drag>>::run(on_drag),
                        On::<Pointer<Click>>::run(on_click),
                        Draggable(entity),
                    ));
            }
        }
    }
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.1, 0.1, 0.4, 0.0),
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.2, 0.4, 0.4, 0.0),
        ..matl.to_owned()
    })),
};

fn make_pickable_recursive(
    commands: &mut Commands,
    root: &Entity,
    entity: &Entity,
    child_query: &Query<&Children>,
    mesh_query: &Query<Entity, (With<Parent>, With<Handle<Mesh>>)>,
) {
    if let Ok(mesh) = mesh_query.get(*entity) {
        commands.entity(mesh).insert((
            PickableBundle::default(),
            RaycastPickTarget::default(),
            HIGHLIGHT_TINT,
            Draggable(*root),
        ));
    }
    if let Ok(children) = child_query.get(*entity) {
        for entity in children.iter() {
            make_pickable_recursive(commands, root, entity, child_query, mesh_query);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block() {
        let mut obj = Block::new(1, Dis2::ZERO)
            .with_tile(Dis2::new(1, 0))
            .with_tile(Dis2::new(2, 0))
            .with_tile(Dis2::new(2, 1))
            .left();
        assert_eq!(
            vec![
                Dis2::new(0, 0),
                Dis2::new(0, 1),
                Dis2::new(0, 2),
                Dis2::new(-1, 2)
            ],
            obj.iter().collect::<Vec<Dis2>>()
        );
        assert_eq!(obj.rotation, Rotation::D90);
        obj.translate(Dis2::X);
        assert_eq!(
            vec![
                Dis2::new(1, 0),
                Dis2::new(1, 1),
                Dis2::new(1, 2),
                Dis2::new(0, 2)
            ],
            obj.iter().collect::<Vec<Dis2>>()
        );
    }

    #[test]
    fn test_rot() {
        let f = Dis2::NEG_Z;
        assert_eq!(f.rotated(Rotation::D0), Rotation::D0.as_discrete());
        assert_eq!(f.rotated(Rotation::D90), Rotation::D90.as_discrete());
        assert_eq!(f.rotated(Rotation::D180), Rotation::D180.as_discrete());
        assert_eq!(f.rotated(Rotation::D270), Rotation::D270.as_discrete());
        let f = Vec3::NEG_Z;
        assert!(
            (Quat::from_rotation_y(Rotation::D0.as_radians()) * f - Rotation::D0.as_vec3())
                .length_squared()
                < 0.1
        );
        assert!(
            (Quat::from_rotation_y(Rotation::D90.as_radians()) * f - Rotation::D90.as_vec3())
                .length_squared()
                < 0.1
        );
        assert!(
            (Quat::from_rotation_y(Rotation::D180.as_radians()) * f - Rotation::D180.as_vec3())
                .length_squared()
                < 0.1
        );
        assert!(
            (Quat::from_rotation_y(Rotation::D270.as_radians()) * f - Rotation::D270.as_vec3())
                .length_squared()
                < 0.1
        );
    }

    #[test]
    fn test_coord() {
        let level = Level::new(4, 6);
        assert_eq!(level.get(5, 1), None);
        assert_eq!(level.get(1, 7), None);
        assert_eq!(
            level.to_discrete(level.offset() + Vec3::new(2.0, 0.0, 1.0)),
            Dis2::new(2, 1)
        );
        assert_eq!(
            level.to_discrete(level.offset() + Vec3::new(3.0, 0.0, 5.0)),
            Dis2::new(3, 5)
        );
        assert!(
            level
                .to_vec3(Dis2::new(3, 5))
                .distance_squared(level.offset() + Vec3::new(3.0, 0.0, 5.0))
                < 0.1
        );
        assert!(
            level
                .to_vec3(Dis2::new(2, 1))
                .distance_squared(level.offset() + Vec3::new(2.0, 0.0, 1.0))
                < 0.1
        );
    }
}
