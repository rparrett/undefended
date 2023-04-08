pub const FOCUSED_BUTTON: Color = Color::rgb(0.25, 0.0, 0.25);
use bevy::prelude::*;
use bevy_ui_navigation::prelude::*;

use crate::GameState;

pub const FOCUSED_HOVERED_BUTTON: Color = Color::rgb(0.35, 0.0, 0.35);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const CONTAINER_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Component)]
pub struct GameUiMarker;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnExit(GameState::MainMenu)));
    }
}

fn setup() {}

pub fn buttons(
    mut interaction_query: Query<
        (&Interaction, &Focusable, &mut BackgroundColor),
        (Or<(Changed<Interaction>, Changed<Focusable>)>, With<Button>),
    >,
) {
    for (interaction, focusable, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                if matches!(focusable.state(), FocusState::Focused) {
                    *color = FOCUSED_HOVERED_BUTTON.into()
                } else {
                    *color = HOVERED_BUTTON.into();
                };
            }
            Interaction::None => {
                if matches!(focusable.state(), FocusState::Focused) {
                    *color = FOCUSED_BUTTON.into()
                } else {
                    *color = NORMAL_BUTTON.into();
                };
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<GameUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
