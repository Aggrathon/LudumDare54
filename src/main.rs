use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_mod_picking::DefaultPickingPlugins;

use camera::CameraMovePlugin;
use cubes::CubePlugin;
use game::GamePlugin;
use levels::LevelManagerPlugin;
use load::LoadPlugin;
use ui::UIPlugin;

mod camera;
mod cubes;
mod game;
mod levels;
mod load;
mod objects;
mod ui;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 4.0f32,
        })
        .add_state::<AppState>()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            EasingsPlugin,
            LoadPlugin,
            CameraMovePlugin,
            GamePlugin,
            CubePlugin,
            LevelManagerPlugin,
            UIPlugin,
        ))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Level,
    Unloading,
}
