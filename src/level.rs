use std::f32::consts::PI;

use bevy::math::vec4;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins)
            // .add_systems(Startup, setup)
            .add_systems(Update, make_scene_draggable);
    }
}

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
// }

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
    pub fn get(&self, x: usize, y: usize) -> Option<&usize> {
        if x >= self.width {
            None
        } else {
            self.floor.get(y * self.width + x)
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut usize> {
        if x >= self.width {
            None
        } else {
            self.floor.get_mut(y * self.width + x)
        }
    }

    pub fn offset(&self) -> (f32, f32) {
        (-(self.width as f32 * 0.5), -(self.height as f32 * 0.5))
    }

    pub fn next_index(&mut self) -> usize {
        self.index += 1;
        self.index
    }

    pub fn try_place(&self, block: &Block, x: usize, y: usize, dir: Direction) -> bool {
        for (x, y) in block.iter_with(x as isize, y as isize, dir) {
            if x < 0 || y < 0 {
                return false;
            }
            match self.get(x as usize, y as usize) {
                Some(index) => {
                    if *index != 0 && *index != block.index {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    pub fn remove(&mut self, block: &Block) {
        for (x, y) in block.iter() {
            *self.get_mut(x as usize, y as usize).unwrap() = 0;
        }
    }

    pub fn place(&mut self, block: &Block) {
        for (x, y) in block.iter() {
            *self.get_mut(x as usize, y as usize).unwrap() = block.index;
        }
    }

    pub fn to_discrete(&self, pos: Vec3) -> (usize, usize) {
        let (mut x, mut y) = self.offset();
        x = pos.x - x;
        y = pos.z - y;
        (x.round() as usize, y.round() as usize)
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    pub fn rotate(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }

    pub fn rotate_by(self, other: Direction) -> Self {
        match other {
            Direction::North => self,
            Direction::West => self.rotate(),
            Direction::South => self.rotate().rotate(),
            Direction::East => self.rotate().rotate().rotate(),
        }
    }

    pub fn as_radians(&self) -> f32 {
        match self {
            Direction::North => 0.0,
            Direction::West => -PI * 0.5,
            Direction::South => PI,
            Direction::East => PI * 0.5,
        }
    }
}

#[derive(Component)]
pub struct Block {
    pub index: usize,
    path: Vec<Direction>,
    rotation: Direction,
    x: usize,
    y: usize,
}

impl Block {
    pub fn new(index: usize, x: usize, y: usize) -> Self {
        Self {
            index,
            path: Vec::new(),
            rotation: Direction::North,
            x,
            y,
        }
    }

    pub fn north(mut self) -> Self {
        self.path.push(Direction::North);
        self
    }

    pub fn west(mut self) -> Self {
        self.path.push(Direction::West);
        self
    }

    pub fn south(mut self) -> Self {
        self.path.push(Direction::South);
        self
    }

    pub fn east(mut self) -> Self {
        self.path.push(Direction::East);
        self
    }

    pub fn rotate_next(mut self) -> Self {
        self.rotation = self.rotation.rotate();
        self
    }

    pub fn translate(&mut self, x: usize, y: usize) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn rotate(&mut self) -> &mut Self {
        self.rotation = self.rotation.rotate();
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (isize, isize)> + '_ {
        self.iter_with(self.x as isize, self.y as isize, self.rotation)
    }

    pub fn iter_with(
        &self,
        x: isize,
        y: isize,
        rotation: Direction,
    ) -> impl Iterator<Item = (isize, isize)> + '_ {
        let mut x = x;
        let mut y = y;
        [(x, y)].into_iter().chain(self.path.iter().map(move |dir| {
            match dir.rotate_by(rotation) {
                Direction::North => y += 1,
                Direction::West => x -= 1,
                Direction::South => y -= 1,
                Direction::East => x += 1,
            };
            (x, y)
        }))
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
                    let mut pos = ray.get_point(dist);
                    let (x, y) = level.offset();
                    pos.x = (pos.x - x).round() + x;
                    pos.z = (pos.z - y).round() + y;
                    let (x, y) = level.to_discrete(pos);
                    if (x != block.x || y != block.y)
                        && level.try_place(&block, x, y, block.rotation)
                    {
                        level.remove(&block);
                        block.translate(x, y);
                        level.place(&block);
                        transform.translation = pos;
                        // TODO place sound
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
) {
    if event.button != PointerButton::Secondary {
        return;
    }
    if let Ok(root) = root_query.get(event.target) {
        if let Ok((mut transform, mut block)) = block_query.get_mut(root.0) {
            let mut dir = block.rotation;
            for _ in 0..3 {
                dir = dir.rotate();
                if level.try_place(&block, block.x, block.y, dir) {
                    level.remove(&block);
                    block.rotation = dir;
                    level.place(&block);
                    transform.rotation = Quat::from_rotation_y(dir.as_radians());
                    // TODO place sound
                    return;
                }
            }
            // TODO: Error sound
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
        base_color: matl.base_color + vec4(0.1, 0.1, 0.4, 0.0),
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
        let obj = Block::new(1, 0, 0).north().north().north().rotate_next();
        assert_eq!(
            vec![(0, 0), (-1, 0), (-2, 0), (-3, 0)],
            obj.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_coord() {
        let level = Level::new(4, 6);
        assert_eq!(level.get(5, 1), None);
        assert_eq!(level.get(1, 7), None);
        let (x, y) = level.offset();
        assert_eq!(level.to_discrete(Vec3::new(x + 2.0, 0.0, y + 2.0)), (2, 2));
        assert_eq!(level.to_discrete(Vec3::new(x + 3.0, 0.0, y + 5.0)), (3, 5));
    }
}
