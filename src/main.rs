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
            brightness: 1.0 / 2.0f32,
        })
        .add_state::<AppState>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Conveyor Chaos   -   Aggrathon   -   Ludum Dare 54".to_string(),
                    resizable: true,
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins,
            EasingsPlugin,
            CameraMovePlugin,
            GamePlugin,
            CubePlugin,
            UIPlugin,
            LevelManagerPlugin,
            LoadPlugin,
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
