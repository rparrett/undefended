use bevy::{audio::AudioSink, prelude::*};
use bevy_ui_navigation::prelude::*;

use crate::{
    loading::{Fonts, Sounds},
    settings::{MusicSetting, SfxSetting},
    ui::{buttons, BUTTON_TEXT, CONTAINER_BACKGROUND, NORMAL_BUTTON, TITLE_TEXT},
    GameState, MusicController,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_systems(
                (
                    sfx_volume,
                    music_volume,
                    button_actions,
                    buttons.after(NavRequestSystem),
                )
                    .in_set(OnUpdate(GameState::MainMenu)),
            )
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuMarker;

#[derive(Component)]
struct PlayButton;
#[derive(Component)]
struct MusicSettingButton;
#[derive(Component)]
struct MusicSettingButtonText;
#[derive(Component)]
struct SfxSettingButton;

#[derive(Component)]
struct SfxSettingButtonText;

fn setup_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(45.0)),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: fonts.main.clone(),
        font_size: 30.0,
        color: BUTTON_TEXT,
    };
    let title_text_style = TextStyle {
        font: fonts.main.clone(),
        font_size: 60.0,
        color: TITLE_TEXT,
    };
    let subtitle_text_style = TextStyle {
        font: fonts.main.clone(),
        font_size: 40.0,
        color: BUTTON_TEXT,
    };

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                background_color: CONTAINER_BACKGROUND.into(),
                ..default()
            },
            MainMenuMarker,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section("TODO", title_text_style).with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Play,
            PlayButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Play", button_text_style.clone()));
        })
        .id();

    let audio_settings_title = commands
        .spawn(
            TextBundle::from_section("Audio", subtitle_text_style).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let sfx_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Sfx,
            SfxSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("SFX {}%", **sfx), button_text_style.clone()),
                SfxSettingButtonText,
            ));
        })
        .id();

    let music_button = commands
        .spawn((
            ButtonBundle {
                style: button_style,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Music,
            MusicSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("Music {}%", **music), button_text_style),
                MusicSettingButtonText,
            ));
        })
        .id();

    commands.entity(container).push_children(&[
        title,
        play_button,
        audio_settings_title,
        sfx_button,
        music_button,
    ]);

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(40.),
                    ..default()
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Px(50.),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..default()
        },
        MainMenuMarker,
    ));
}

#[derive(Component)]
enum MenuButton {
    Play,
    Sfx,
    Music,
}

fn button_actions(
    buttons: Query<&MenuButton>,
    mut events: EventReader<NavEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut music_setting: ResMut<MusicSetting>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<SfxSettingButtonText>>,
        Query<&mut Text, With<MusicSettingButtonText>>,
    )>,
    mut sfx_setting: ResMut<SfxSetting>,
) {
    for button in events.nav_iter().activated_in_query(&buttons) {
        match button {
            MenuButton::Play => {
                next_state.set(GameState::Playing);
            }
            MenuButton::Sfx => {
                if **sfx_setting == 0 {
                    **sfx_setting = 100;
                } else {
                    **sfx_setting -= 10;
                }

                for mut text in text_queries.p0().iter_mut() {
                    text.sections[0].value = format!("SFX {}%", **sfx_setting);
                }
            }
            MenuButton::Music => {
                if **music_setting == 0 {
                    **music_setting = 100;
                } else {
                    **music_setting -= 10;
                }

                for mut text in text_queries.p1().iter_mut() {
                    text.sections[0].value = format!("Music {}%", **music_setting);
                }
            }
        }
    }
}

fn sfx_volume(sfx_setting: Res<SfxSetting>, audio: Res<Audio>, game_audio: Res<Sounds>) {
    // Do not run when SfxSetting is first added by SavePlugin
    if !sfx_setting.is_changed() || sfx_setting.is_added() {
        return;
    }

    audio.play_with_settings(
        game_audio.build.clone(),
        PlaybackSettings::ONCE.with_volume(**sfx_setting as f32 / 100.),
    );
}

fn music_volume(
    music_setting: Res<MusicSetting>,
    audio_sinks: Res<Assets<AudioSink>>,
    controller: Option<Res<MusicController>>,
) {
    // Do not run when MusicSetting is first added by SavePlugin
    if !music_setting.is_changed() || music_setting.is_added() {
        return;
    }

    if let Some(controller) = controller {
        if let Some(sink) = audio_sinks.get(&controller.0) {
            sink.set_volume(**music_setting as f32 / 100.)
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
