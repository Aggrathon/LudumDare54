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
    pub fn get(&self, i: usize, j: usize) -> Option<&usize> {
        self.floor.get(j * self.width + i)
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut usize> {
        self.floor.get_mut(j * self.width + i)
    }

    pub fn offset(&self) -> (f32, f32) {
        (-(self.width as f32 * 0.5), -(self.height as f32 * 0.5))
    }

    pub fn next_index(&mut self) -> usize {
        self.index += 1;
        self.index
    }
}

#[derive(Component)]
pub struct MakeSceneDraggable;

fn on_drag(
    event: Listener<Pointer<Drag>>,
    mut query: Query<(&mut Transform, &Draggable)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    level: ResMut<Level>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    if let Ok((_, root)) = query.get(event.target) {
        if let Ok((mut transform, _)) = query.get_mut(root.0) {
            let (camera, camera_transform) = camera.single();
            if let Some(ray) =
                camera.viewport_to_world(camera_transform, event.pointer_location.position)
            {
                if let Some(dist) = ray.intersect_plane(transform.translation, Vec3::Y) {
                    let mut pos = ray.get_point(dist);
                    let (x, y) = level.offset();
                    pos.x = (pos.x - x).round() + x;
                    pos.z = (pos.z - y).round() + y;
                    // TODO check if position is valid
                    transform.translation = pos;
                }
            }
        }
    }
}

fn on_click(event: Listener<Pointer<Click>>, mut query: Query<(&mut Transform, &Draggable)>) {
    if event.button != PointerButton::Secondary {
        return;
    }
    if let Ok((_, root)) = query.get(event.target) {
        if let Ok((mut transform, _)) = query.get_mut(root.0) {
            // TODO check if rotation is valid
            transform.rotate_y(PI * 0.5);
        }
    }
}

#[derive(Component)]
struct Draggable(Entity);

fn make_scene_draggable(
    mut commands: Commands,
    mut query: Query<Entity, (With<Children>, With<MakeSceneDraggable>)>,
    mesh_query: Query<Entity, (With<Parent>, With<Handle<Mesh>>)>,
    child_query: Query<&Children>,
) {
    for entity in query.iter_mut() {
        dbg!(entity);
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
