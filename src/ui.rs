use bevy::{prelude::*, ui::UiSystem, utils::Duration};
use bevy_alt_ui_navigation_lite::prelude::*;
use bevy_dolly::system::DollyUpdateSet;

use crate::{
    loading::{Fonts, Images},
    map::ItemSpawner,
    settings::DifficultySetting,
    tower::Ammo,
    waves::{WaveState, Waves},
    GameState, Lives, MainCamera,
};

pub const FOCUSED_BUTTON: Srgba = Srgba::rgb(0.25, 0.0, 0.25);
pub const FOCUSED_HOVERED_BUTTON: Srgba = Srgba::rgb(0.35, 0., 0.35);
pub const NORMAL_BUTTON: Srgba = Srgba::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Srgba = Srgba::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Srgba = Srgba::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Srgba = Srgba::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Srgba = bevy::color::palettes::css::DEEP_PINK;
pub const UI_TEXT: Srgba = bevy::color::palettes::css::DEEP_PINK;
pub const ALT_TEXT: Srgba = Srgba::rgb(0.9, 0.9, 0.9);
pub const CONTAINER_BACKGROUND: Srgba = Srgba::rgb(0.1, 0.1, 0.1);
pub const OVERLAY: Srgba = Srgba::new(0.0, 0.0, 0.0, 0.6);
pub const AMMO: Srgba = bevy::color::palettes::css::YELLOW;
pub const AMMO_EMPTY: Srgba = bevy::color::palettes::css::RED;
pub const SPAWNER_TIMER: Srgba = bevy::color::palettes::css::YELLOW;

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
        app.add_systems(
            Update,
            (
                update_waves,
                update_wave_timer,
                update_wave_stats,
                update_ammo,
                spawn_ammo,
                update_item_spawners,
                spawn_item_spawners,
                update_lives,
            )
                .distributive_run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            follow
                .after(DollyUpdateSet)
                .before(UiSystem::Layout)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::MainMenu), setup)
        .add_systems(OnExit(GameState::MainMenu), setup_lives);
    }
}

fn setup(mut commands: Commands, fonts: Res<Fonts>) {
    let text_style = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 16.,
            ..default()
        },
        TextColor(UI_TEXT.into()),
    );
    let text_style_alt = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 16.,
            ..default()
        },
        TextColor(ALT_TEXT.into()),
    );

    commands
        .spawn((
            Name::new("WaveInfoContainer"),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(165.),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
            BackgroundColor(OVERLAY.into()),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("WaveNumberContainer"),
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("WAVE:"), text_style.clone()));
                    parent
                        .spawn((WaveText, Text::new("?"), text_style_alt.clone()))
                        .with_child((TextSpan::new("/"), text_style.clone()))
                        .with_child((TextSpan::new("?"), text_style.clone()));
                });

            parent
                .spawn((
                    Name::new("WaveTimerContainer"),
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("TIME:"), text_style.clone()));
                    parent
                        .spawn((WaveTimerText, Text::new("--"), text_style_alt.clone()))
                        .with_child((TextSpan::new("s"), text_style.clone()));
                });

            parent
                .spawn((
                    Name::new("WaveStatsContainer"),
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("STATS:"), text_style.clone()));
                    parent
                        .spawn((WaveStatsText, Text::new("?"), text_style_alt.clone()))
                        .with_child((TextSpan::new("x "), text_style.clone()))
                        .with_child((TextSpan::new("?"), text_style_alt.clone()))
                        .with_child((TextSpan::new("HP"), text_style.clone()));
                });
        });
}

fn setup_lives(mut commands: Commands, lives: Res<Lives>, images: Res<Images>) {
    commands
        .spawn((
            LivesContainer,
            Name::new("LivesContainer"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
            BackgroundColor(OVERLAY.into()),
        ))
        .with_children(|parent| {
            for i in 0..lives.0 {
                let padding = if i + 1 == lives.0 { 0.0 } else { 5.0 };

                parent.spawn((
                    ImageNode {
                        image: images.heart.clone().into(),
                        ..default()
                    },
                    Node {
                        margin: UiRect::right(Val::Px(padding)),
                        max_width: Val::Px(20.0),
                        max_height: Val::Px(20.0),
                        ..default()
                    },
                ));
            }
        });
}

fn update_lives(
    lives: Res<Lives>,
    container_query: Query<&Children, With<LivesContainer>>,
    mut image_query: Query<&mut Node>,
) {
    if !lives.is_changed() {
        return;
    }

    for children in container_query.iter() {
        let mut i = 0;
        for child in children {
            if let Ok(mut node) = image_query.get_mut(*child) {
                node.display = if i + 1 > lives.0 {
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
            Interaction::Pressed => {
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
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(100.),
                    height: Val::Px(20.),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                GlobalZIndex(-1),
                // TODO I think we can opt out of UI rounding, right?
                // We should do this for FollowInWorld.
                FollowInWorld(entity),
            ))
            .with_children(|parent| {
                parent.spawn((
                    AmmoText(entity),
                    Text::new(format!("{}/{}", ammo.current, ammo.max)),
                    TextFont {
                        font: fonts.main.clone(),
                        font_size: 16.,
                        ..default()
                    },
                    TextColor(AMMO.into()),
                ));
            });
    }
}

fn update_ammo(
    mut query: Query<(&mut Text, &mut TextColor, &AmmoText)>,
    ammo_query: Query<&Ammo, Changed<Ammo>>,
) {
    for (mut text, mut text_color, entity) in query.iter_mut() {
        let Ok(ammo) = ammo_query.get(entity.0) else {
            continue;
        };

        if ammo.current == 0 {
            text_color.0 = AMMO_EMPTY.into();
        } else {
            text_color.0 = AMMO.into();
        }

        text.0 = format!("{}/{}", ammo.current, ammo.max);
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
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(100.),
                    height: Val::Px(20.),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                GlobalZIndex(-1),
                FollowInWorld(entity),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ItemSpawnerText(entity),
                    Text::new("?"),
                    TextFont {
                        font: fonts.main.clone(),
                        font_size: 16.,
                        ..default()
                    },
                    TextColor(SPAWNER_TIMER.into()),
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
            continue;
        };

        if item_spawner.timer.remaining() == Duration::ZERO && item_spawner.spawned == 1 {
            text.0 = format!("{}", item_spawner.item);
            *visibility = Visibility::Inherited;
        } else if item_spawner.timer.remaining() == Duration::ZERO {
            *visibility = Visibility::Hidden;
        } else {
            text.0 = format!("0:{:0>2.0}", item_spawner.timer.remaining_secs());
            *visibility = Visibility::Inherited;
        }
    }
}

fn follow(
    mut query: Query<(&mut Node, &FollowInWorld)>,
    world_query: Query<&GlobalTransform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (mut node, follow) in query.iter_mut() {
        let Ok(world) = world_query.get(follow.0) else {
            continue;
        };

        let Ok(viewport) =
            camera.world_to_viewport(camera_transform, world.translation() + Vec3::Y * 2.0)
        else {
            continue;
        };

        let width = match node.width {
            Val::Px(px) => px,
            _ => continue,
        };

        node.left = Val::Px((viewport.x - width / 2.).round());
        node.top = Val::Px(viewport.y);
    }
}

fn update_waves(query: Query<Entity, With<WaveText>>, waves: Res<Waves>, mut writer: TextUiWriter) {
    for text in &query {
        let wave = if waves.current == waves.waves.len() {
            waves.current
        } else {
            waves.current + 1
        };

        *writer.text(text, 0) = format!("{}", wave);
        *writer.text(text, 2) = format!("{}", waves.waves.len());
    }
}

fn update_wave_timer(
    query: Query<Entity, With<WaveTimerText>>,
    wave_state: Res<WaveState>,
    waves: Res<Waves>,
    mut writer: TextUiWriter,
) {
    for text in &query {
        if waves.current == waves.waves.len() {
            *writer.text(text, 0) = "--".to_string();
            writer.text(text, 1).clear();
        } else if wave_state.delay_timer.remaining() == Duration::ZERO {
            *writer.text(text, 0) = "NOW!".to_string();
            writer.text(text, 1).clear();
        } else {
            *writer.text(text, 0) = format!("{:.1}", wave_state.delay_timer.remaining_secs());
            *writer.text(text, 1) = "s".to_string();
        };
    }
}

fn update_wave_stats(
    query: Query<Entity, With<WaveStatsText>>,
    waves: Res<Waves>,
    difficulty: Res<DifficultySetting>,
    mut writer: TextUiWriter,
) {
    let Some(current) = waves.current() else {
        return;
    };

    let extra_hp = match *difficulty {
        DifficultySetting::Normal => 0,
        DifficultySetting::Hard => 1,
        DifficultySetting::Extra => 2,
    };

    for text in &query {
        *writer.text(text, 0) = format!("{}", current.num);
        *writer.text(text, 2) = format!("{}", current.hp + extra_hp);
    }
}
