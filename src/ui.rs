use bevy::{prelude::*, utils::Duration};
use bevy_ui_navigation::prelude::*;

use crate::{
    loading::{Fonts, Images},
    map::ItemSpawner,
    settings::DifficultySetting,
    tower::Ammo,
    waves::{WaveState, Waves},
    GameState, Lives, MainCamera,
};

pub const FOCUSED_BUTTON: Color = Color::rgb(0.25, 0.0, 0.25);
pub const FOCUSED_HOVERED_BUTTON: Color = Color::rgb(0.35, 0., 0.35);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::PINK;
pub const UI_TEXT: Color = Color::PINK;
pub const ALT_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const CONTAINER_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);
pub const OVERLAY: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

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

#[derive(Component)]
pub struct LivesContainer;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_waves.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                update_wave_timer.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_wave_stats.v(in_state(GameState::Playing)))
            .add_systems(Update, follow.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_ammo.run_if(in_state(GameState::Playing)))
            .add_systems(Update, spawn_ammo.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                update_item_spawners.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                spawn_item_spawners.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::MainMenu), setup)
            .add_systems(OnExit(GameState::MainMenu), setup_lives)
            .add_systems(Update, update_lives.run_if(in_state(GameState::Playing)));
    }
}

fn setup(mut commands: Commands, fonts: Res<Fonts>) {
    let text_style = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: UI_TEXT,
    };
    let text_style_alt = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: ALT_TEXT,
    };

    commands
        .spawn((
            Name::new("WaveInfoContainer"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(165.),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
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
                            text: Text::from_sections([
                                TextSection {
                                    value: "?".to_string(),
                                    style: text_style_alt.clone(),
                                },
                                TextSection {
                                    value: "/".to_string(),
                                    style: text_style.clone(),
                                },
                                TextSection {
                                    value: "?".to_string(),
                                    style: text_style.clone(),
                                },
                            ]),
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
                            value: "TIME:".to_string(),
                            style: text_style.clone(),
                        }]),
                        ..default()
                    });
                    parent.spawn((
                        WaveTimerText,
                        TextBundle {
                            text: Text::from_sections([
                                TextSection {
                                    value: "--".to_string(),
                                    style: text_style_alt.clone(),
                                },
                                TextSection {
                                    value: "s".to_string(),
                                    style: text_style.clone(),
                                },
                            ]),
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
                            text: Text::from_sections([
                                TextSection {
                                    value: "?".to_string(),
                                    style: text_style_alt.clone(),
                                },
                                TextSection {
                                    value: "x ".to_string(),
                                    style: text_style.clone(),
                                },
                                TextSection {
                                    value: "?".to_string(),
                                    style: text_style_alt.clone(),
                                },
                                TextSection {
                                    value: "HP".to_string(),
                                    style: text_style.clone(),
                                },
                            ]),
                            ..default()
                        },
                    ));
                });
        });
}

fn setup_lives(mut commands: Commands, lives: Res<Lives>, images: Res<Images>) {
    commands
        .spawn((
            LivesContainer,
            Name::new("LivesContainer"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: OVERLAY.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..lives.0 {
                let padding = if i + 1 == lives.0 { 0.0 } else { 5.0 };

                parent.spawn(ImageBundle {
                    image: images.heart.clone().into(),
                    style: Style {
                        margin: UiRect::right(Val::Px(padding)),
                        max_width: Val::Px(20.0),
                        max_height: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });
            }
        });
}

fn update_lives(
    lives: Res<Lives>,
    container_query: Query<&Children, With<LivesContainer>>,
    mut image_query: Query<&mut Style>,
) {
    if !lives.is_changed() {
        return;
    }

    for children in container_query.iter() {
        let mut i = 0;
        for child in children {
            if let Ok(mut style) = image_query.get_mut(*child) {
                style.display = if i + 1 > lives.0 {
                    Display::None
                } else {
                    Display::Flex
                };

                i += 1;
            }
        }
    }
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
                        width: Val::Px(100.),
                        height: Val::Px(20.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    z_index: ZIndex::Global(-1),
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
                        width: Val::Px(100.),
                        height: Val::Px(20.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    z_index: ZIndex::Global(-1),
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
    mut query: Query<(&mut Text, &mut Visibility, &ItemSpawnerText)>,
    item_spawner_query: Query<&ItemSpawner, Changed<ItemSpawner>>,
) {
    for (mut text, mut visibility, entity) in query.iter_mut() {
        let Ok(item_spawner) = item_spawner_query.get(entity.0) else {
            continue
        };

        if item_spawner.timer.remaining() == Duration::ZERO && item_spawner.spawned == 1 {
            text.sections[0].value = format!("{}", item_spawner.item);
            *visibility = Visibility::Inherited;
        } else if item_spawner.timer.remaining() == Duration::ZERO {
            *visibility = Visibility::Hidden;
        } else {
            text.sections[0].value = format!("0:{:0>2.0}", item_spawner.timer.remaining_secs());
            *visibility = Visibility::Inherited;
        }
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

        text.sections[0].value = format!("{}", wave);
        text.sections[2].value = format!("{}", waves.waves.len());
    }
}

fn update_wave_timer(
    mut query: Query<&mut Text, With<WaveTimerText>>,
    wave_state: Res<WaveState>,
    waves: Res<Waves>,
) {
    for mut text in query.iter_mut() {
        if waves.current == waves.waves.len() {
            text.sections[0].value = "--".to_string();
            text.sections[1].value.clear();
        } else if wave_state.delay_timer.remaining() == Duration::ZERO {
            text.sections[0].value = "NOW!".to_string();
            text.sections[1].value.clear();
        } else {
            text.sections[0].value = format!("{:.1}", wave_state.delay_timer.remaining_secs());
            text.sections[1].value = "s".to_string();
        };
    }
}

fn update_wave_stats(
    mut query: Query<&mut Text, With<WaveStatsText>>,
    waves: Res<Waves>,
    difficulty: Res<DifficultySetting>,
) {
    let Some(current) = waves.current() else {
        return
    };

    let extra_hp = match *difficulty {
        DifficultySetting::Normal => 0,
        DifficultySetting::Hard => 1,
        DifficultySetting::Extra => 2,
    };

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{}", current.num);
        text.sections[2].value = format!("{}", current.hp + extra_hp);
    }
}
