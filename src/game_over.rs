use bevy::prelude::*;
use bevy_alt_ui_navigation_lite::prelude::*;

use crate::{
    loading::Fonts,
    ui::{buttons, BUTTON_TEXT, CONTAINER_BACKGROUND, NORMAL_BUTTON, TITLE_TEXT},
    GameState, Won,
};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn)
            .add_systems(
                Update,
                (button_actions, buttons.after(NavRequestSystem))
                    .run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

#[derive(Component)]
struct GameOverMarker;
#[derive(Component)]
struct PlayAgainButton;
#[derive(Component)]
enum GameOverButton {
    PlayAgain,
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>, won: Res<Won>) {
    let title_text_style = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 50.0,
            ..default()
        },
        TextColor(TITLE_TEXT.into()),
    );
    let button_node = Node {
        width: Val::Px(250.0),
        height: Val::Px(45.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 25.0,
            ..default()
        },
        TextColor(BUTTON_TEXT.into()),
    );

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            GameOverMarker,
        ))
        .id();

    let container = commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            BackgroundColor(CONTAINER_BACKGROUND.into()),
        ))
        .id();

    let title = commands
        .spawn((
            Text::new(if won.0 { "YOU WIN!" } else { "GAME OVER!" }),
            title_text_style,
            Node {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let play_again = commands
        .spawn((
            Button,
            button_node,
            BackgroundColor(NORMAL_BUTTON.into()),
            Focusable::default(),
            GameOverButton::PlayAgain,
            PlayAgainButton,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("PLAY AGAIN"), button_text_style.clone()));
        })
        .id();

    commands.entity(root).add_children(&[container]);

    commands
        .entity(container)
        .add_children(&[title, play_again]);
}

fn button_actions(
    buttons: Query<&GameOverButton>,
    mut events: EventReader<NavEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for button in events.nav_iter().activated_in_query(&buttons) {
        match button {
            GameOverButton::PlayAgain => {
                next_state.set(GameState::MainMenu);
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<GameOverMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
