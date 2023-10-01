use bevy::prelude::*;

use crate::levels::{LevelEntity, LevelState};
use crate::AppState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_interaction)
            .add_systems(
                Update,
                (next_level_button).run_if(in_state(AppState::Level)),
            )
            .add_systems(
                OnEnter(AppState::Loading),
                setup_main_menu.run_if(in_state(LevelState::MainMenu)),
            );
    }
}

#[derive(Component)]
struct NextLevelButton;

const NORMAL_BUTTON: Color = Color::WHITE;
const HOVERED_BUTTON: Color = Color::rgb(0.85, 0.95, 1.00);
const PRESSED_BUTTON: Color = Color::rgb(0.9, 1.00, 1.00);
const BORDER_BUTTON: Color = Color::BLACK;

fn button_interaction(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn next_level_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NextLevelButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in interaction_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            next_state.set(AppState::Unloading);
            return;
        }
    }
}

fn setup_main_menu(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((
        LevelEntity,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(10.0),
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|parent| {
        let mut text = TextBundle::from_section(
            "Conveyor Chaos",
            TextStyle {
                font: asset_server.load("fonts/Tourney-SemiBold.ttf"),
                font_size: 128.0,
                color: Color::BLACK,
            },
        );
        text.background_color = Color::rgba(1.0, 1.0, 1.0, 0.5).into();
        parent.spawn(text);
        parent
            .spawn((
                NextLevelButton,
                ButtonBundle {
                    style: Style {
                        width: Val::Px(220.0),
                        height: Val::Px(90.0),
                        border: UiRect::all(Val::Px(4.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BORDER_BUTTON.into(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/Tourney-Regular.ttf"),
                        font_size: 64.0,
                        color: Color::BLACK,
                    },
                ));
            });
    });
}
