use std::f32::consts::*;

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::load::LoadLevel;
use crate::ui::{ShowDialog, ShowVictory};
use crate::AppState;

pub struct LevelManagerPlugin;

impl Plugin for LevelManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<LevelState>()
            .add_systems(Update, (animate_sun_direction, skip_level))
            .add_systems(OnEnter(AppState::Unloading), unload_level)
            .add_systems(OnEnter(AppState::Loading), (spawn_sun, load_level));
    }
}

#[derive(Component)]
pub struct LevelEntity;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum LevelState {
    #[default]
    MainMenu,
    Level00,
    Level01,
    Level02,
    Level03,
    Test,
}

impl LevelState {
    pub fn get_path(&self) -> &'static str {
        match self {
            LevelState::MainMenu => "levels/main_menu.ron",
            LevelState::Level00 => "levels/level_00.ron",
            LevelState::Level01 => "levels/level_01.ron",
            LevelState::Level02 => "levels/level_02.ron",
            LevelState::Level03 => "levels/level_03.ron",
            LevelState::Test => "levels/test.ron",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            LevelState::MainMenu => LevelState::Level00,
            LevelState::Level00 => LevelState::Level01,
            LevelState::Level01 => LevelState::Level02,
            LevelState::Level02 => LevelState::Level03,
            LevelState::Level03 => LevelState::MainMenu,
            LevelState::Test => LevelState::MainMenu,
        }
    }
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_xyz(0.0, 3.0, 0.0)
                .looking_at(Vec3::new(1.0, 0.0, 1.0), Vec3::Y),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder::default().build(),
            ..default()
        },
        LevelEntity,
    ));
}

fn unload_level(
    query: Query<Entity, (Without<Parent>, With<LevelEntity>)>,
    mut cmds: Commands,
    level: Res<State<LevelState>>,
    mut next_level: ResMut<NextState<LevelState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for entity in query.iter() {
        cmds.entity(entity).despawn_recursive();
    }
    next_level.set(level.next());
    next_state.set(AppState::Loading);
}

fn load_level(
    level: Res<State<LevelState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut dialog: EventWriter<ShowDialog>,
    mut victory: ResMut<ShowVictory>,
) {
    commands.insert_resource(LoadLevel(asset_server.load(level.get_path())));
    match **level {
        LevelState::MainMenu => victory.disable(),
        LevelState::Level00 => {
            dialog.send(ShowDialog("Use WASD/arrows + QE to look around.\nDrag conveyor belts with your mouse.\nRight click to rotate them (if there is enough space).".to_string()));
        }
        _ => {}
    };
}

fn skip_level(mut keys: ResMut<Input<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::K) {
        keys.reset(KeyCode::K);
        state.set(AppState::Unloading);
    }
}

fn animate_sun_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        *transform = Transform::from_xyz(0.0, 3.0, 0.0).looking_at(
            Vec3::new(
                f32::sin(time.elapsed_seconds() * PI / 20.0),
                0.0,
                f32::cos(time.elapsed_seconds() * PI / 20.0),
            ),
            Vec3::Y,
        );
    }
}
