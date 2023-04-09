pub const FOCUSED_BUTTON: Color = Color::rgb(0.25, 0.0, 0.25);
use bevy::prelude::*;
use bevy_ui_navigation::prelude::*;

use crate::{loading::Fonts, tower::Ammo, GameState, MainCamera};

pub const FOCUSED_HOVERED_BUTTON: Color = Color::rgb(0.35, 0.0, 0.35);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::rgb(0.35, 0.0, 0.35);
pub const CONTAINER_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Component)]
pub struct GameUiMarker;

#[derive(Component)]
pub struct FollowInWorld(Entity);

#[derive(Component)]
pub struct AmmoText(Entity);

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow.in_set(OnUpdate(GameState::Playing)))
            .add_system(update_ammo.in_set(OnUpdate(GameState::Playing)))
            .add_system(spawn_ammo.in_set(OnUpdate(GameState::Playing)))
            .add_system(setup.in_schedule(OnExit(GameState::MainMenu)));
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

fn spawn_ammo(
    mut commands: Commands,
    query: Query<(Entity, &Ammo), Added<Ammo>>,
    fonts: Res<Fonts>,
) {
    for (entity, ammo) in query.iter() {
        commands
            .spawn((
                Name::new("AmmoDisplay"),
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        size: Size::new(Val::Px(100.), Val::Px(20.)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                FollowInWorld(entity),
            ))
            .with_children(|parent| {
                parent.spawn((
                    AmmoText(entity),
                    TextBundle {
                        text: Text::from_section(
                            format!("{}/{}", ammo.current, ammo.max),
                            TextStyle {
                                font: fonts.main.clone(),
                                font_size: 20.,
                                color: Color::YELLOW,
                            },
                        ),
                        ..default()
                    },
                ));
            });
    }
}

fn update_ammo(mut query: Query<(&mut Text, &AmmoText)>, ammo_query: Query<&Ammo, Changed<Ammo>>) {
    for (mut text, entity) in query.iter_mut() {
        let Ok(ammo) = ammo_query.get(entity.0) else {
            continue
        };

        if ammo.current == 0 {
            text.sections[0].style.color = Color::RED;
        } else {
            text.sections[0].style.color = Color::YELLOW;
        }

        text.sections[0].value = format!("{}/{}", ammo.current, ammo.max);
    }
}

fn follow(
    mut query: Query<(&mut Style, &FollowInWorld)>,
    world_query: Query<&GlobalTransform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (mut style, follow) in query.iter_mut() {
        let Ok(world) = world_query.get(follow.0) else {
            continue
        };

        let Some(viewport) = camera.world_to_viewport(camera_transform, world.translation() + Vec3::Y * 2.0) else {
            continue;
        };

        style.position = UiRect {
            left: Val::Px(viewport.x).try_sub(style.size.width / 2.).unwrap(),
            bottom: Val::Px(viewport.y),
            ..default()
        };
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<GameUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
