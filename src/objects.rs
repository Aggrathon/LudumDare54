use bevy::prelude::*;

use crate::cubes::CubeRouter;
use crate::game::{Block, Dis2, Level, MakeSceneDraggable, Rotation};
use crate::levels::LevelEntity;

pub struct BeltBuilder {
    tiles: Vec<Dis2>,
    scenes: Vec<SceneBundle>,
    route: Vec<Vec3>,
    dir: Rotation,
    pos: Vec3,
    dis: Dis2,
}

impl BeltBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            scenes: Vec::new(),
            route: vec![Vec3::new(0.0, 0.0, 0.5)],
            dir: Rotation::D0,
            pos: Vec3::ZERO,
            dis: Dis2::ZERO,
        }
    }

    pub fn forward(mut self, asset_server: &Res<AssetServer>) -> Self {
        self.scenes.push(SceneBundle {
            scene: asset_server.load("models/belt.glb#Scene0"),
            transform: Transform::from_translation(self.pos)
                .with_rotation(Quat::from_rotation_y(-self.dir.as_radians())),
            ..Default::default()
        });
        self.route.push(self.pos + self.dir.as_vec3() * 0.5);
        self.tiles.push(self.dis);
        self.pos += self.dir.as_vec3();
        self.dis += self.dir.as_discrete();
        self
    }

    pub fn left(mut self, asset_server: &Res<AssetServer>) -> Self {
        self.scenes.push(SceneBundle {
            scene: asset_server.load("models/beltL.glb#Scene0"),
            transform: Transform::from_translation(self.pos)
                .with_rotation(Quat::from_rotation_y(self.dir.as_radians())),
            ..Default::default()
        });
        self.dir = self.dir.left();
        self.route.push(self.pos);
        self.route.push(self.pos + self.dir.as_vec3() * 0.5);
        self.tiles.push(self.dis);
        self.pos += self.dir.as_vec3();
        self.dis += self.dir.as_discrete();
        self
    }

    pub fn right(mut self, asset_server: &Res<AssetServer>) -> Self {
        self.scenes.push(SceneBundle {
            scene: asset_server.load("models/beltR.glb#Scene0"),
            transform: Transform::from_translation(self.pos)
                .with_rotation(Quat::from_rotation_y(self.dir.as_radians())),
            ..Default::default()
        });
        self.dir = self.dir.right();
        self.route.push(self.pos);
        self.route.push(self.pos + self.dir.as_vec3() * 0.5);
        self.tiles.push(self.dis);
        self.pos += self.dir.as_vec3();
        self.dis += self.dir.as_discrete();
        self
    }

    pub fn build(mut self, dis: Dis2, pos: Vec3, level: &mut Level, cmds: &mut Commands) {
        let mut block = Block::new(level.next_index(), dis);
        block.tiles = self.tiles;
        level.place_unchecked(&block);
        self.route.iter_mut().for_each(|v| v.y += 1.0);
        cmds.spawn((
            SpatialBundle::from_transform(Transform::from_translation(pos)),
            MakeSceneDraggable(None),
            CubeRouter(self.route),
            block,
            LevelEntity,
        ))
        .with_children(|p| {
            for scene in self.scenes.into_iter() {
                p.spawn((MakeSceneDraggable(Some(p.parent_entity())), scene));
            }
        });
    }
}
