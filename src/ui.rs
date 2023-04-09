pub const FOCUSED_BUTTON: Color = Color::rgb(0.25, 0.0, 0.25);
use bevy::{prelude::*, utils::Duration};
use bevy_ui_navigation::prelude::*;

use crate::{
    loading::Fonts,
    map::ItemSpawner,
    tower::Ammo,
    waves::{WaveState, Waves},
    GameState, MainCamera,
};

pub const FOCUSED_HOVERED_BUTTON: Color = Color::rgb(0.35, 0.0, 0.35);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::rgb(0.35, 0.0, 0.35);
pub const CONTAINER_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);
pub const OVERLAY: Color = Color::rgba(0.0, 0.0, 0.0, 0.5);

#[derive(Component)]
pub struct GameUiMarker;

#[derive(Component)]
pub struct FollowInWorld(Entity);

#[derive(Component)]
pub struct AmmoText(Entity);

#[derive(Component)]
pub struct ItemSpawnerText(Entity);

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct WaveTimerText;

#[derive(Component)]
pub struct WaveStatsText;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_waves.in_set(OnUpdate(GameState::Playing)))
            .add_system(update_wave_timer.in_set(OnUpdate(GameState::Playing)))
            .add_system(update_wave_stats.in_set(OnUpdate(GameState::Playing)))
            .add_system(follow.in_set(OnUpdate(GameState::Playing)))
            .add_system(update_ammo.in_set(OnUpdate(GameState::Playing)))
            .add_system(spawn_ammo.in_set(OnUpdate(GameState::Playing)))
            .add_system(update_item_spawners.in_set(OnUpdate(GameState::Playing)))
            .add_system(spawn_item_spawners.in_set(OnUpdate(GameState::Playing)))
            .add_system(setup.in_schedule(OnExit(GameState::MainMenu)));
    }
}

fn setup(mut commands: Commands, fonts: Res<Fonts>) {
    let text_style = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: Color::PINK,
    };

    commands
        .spawn((
            Name::new("WaveInfoContainer"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    size: Size::width(Val::Px(180.)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(0.0),
                        right: Val::Px(0.0),
                        ..default()
                    },
                    padding: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: OVERLAY.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("WaveNumberContainer"),
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_sections([TextSection {
                            value: "WAVE:".to_string(),
                            style: text_style.clone(),
                        }]),
                        ..default()
                    });
                    parent.spawn((
                        WaveText,
                        TextBundle {
                            text: Text::from_sections([TextSection {
                                value: "?".to_string(),
                                style: text_style.clone(),
                            }]),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    Name::new("WaveTimerContainer"),
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_sections([TextSection {
                            value: "NEXT:".to_string(),
                            style: text_style.clone(),
                        }]),
                        ..default()
                    });
                    parent.spawn((
                        WaveTimerText,
                        TextBundle {
                            text: Text::from_sections([TextSection {
                                value: "?".to_string(),
                                style: text_style.clone(),
                            }]),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    Name::new("WaveStatsContainer"),
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_sections([TextSection {
                            value: "STATS:".to_string(),
                            style: text_style.clone(),
                        }]),
                        ..default()
                    });
                    parent.spawn((
                        WaveStatsText,
                        TextBundle {
                            text: Text::from_sections([TextSection {
                                value: "?".to_string(),
                                style: text_style.clone(),
                            }]),
                            ..default()
                        },
                    ));
                });
        });
}

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

fn spawn_item_spawners(
    mut commands: Commands,
    query: Query<Entity, Added<ItemSpawner>>,
    fonts: Res<Fonts>,
) {
    for entity in query.iter() {
        commands
            .spawn((
                Name::new("ItemSpawnerDisplay"),
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
                    ItemSpawnerText(entity),
                    TextBundle {
                        text: Text::from_section(
                            "?".to_string(),
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

fn update_item_spawners(
    mut query: Query<(&mut Text, &ItemSpawnerText)>,
    item_spawner_query: Query<&ItemSpawner, Changed<ItemSpawner>>,
) {
    for (mut text, entity) in query.iter_mut() {
        let Ok(item_spawner) = item_spawner_query.get(entity.0) else {
            continue
        };

        if item_spawner.timer.remaining() == Duration::ZERO {
            text.sections[0].style.color = Color::NONE;
        } else {
            text.sections[0].style.color = Color::PINK;
        }

        text.sections[0].value = format!("0:{:0>2.0}", item_spawner.timer.remaining_secs());
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

fn update_waves(mut query: Query<&mut Text, With<WaveText>>, waves: Res<Waves>) {
    for mut text in query.iter_mut() {
        let wave = if waves.current == waves.waves.len() {
            waves.current
        } else {
            waves.current + 1
        };

        text.sections[0].value = format!("{}/{}", wave, waves.waves.len());
    }
}

fn update_wave_timer(mut query: Query<&mut Text, With<WaveTimerText>>, wave_state: Res<WaveState>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = if wave_state.delay_timer.remaining() == Duration::ZERO {
            "NOW!".to_string()
        } else {
            format!("{:.1}", wave_state.delay_timer.remaining_secs())
        };
    }
}

fn update_wave_stats(mut query: Query<&mut Text, With<WaveStatsText>>, waves: Res<Waves>) {
    let Some(current) = waves.current() else {
        return
    };

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{}x {}HP", current.num, current.hp);
    }
}
