use bevy::{audio::AudioSink, prelude::*};
use bevy_ui_navigation::prelude::*;

use crate::{
    loading::{Fonts, Sounds},
    settings::{DifficultySetting, MusicSetting, SfxSetting},
    ui::{
        buttons, ALT_TEXT, BUTTON_TEXT, CONTAINER_BACKGROUND, NORMAL_BUTTON, TITLE_TEXT, UI_TEXT,
    },
    GameState, MusicController,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_menu)
            .add_systems(
                Update,
                (
                    sfx_volume,
                    music_volume,
                    button_actions,
                    buttons.after(NavRequestSystem),
                )
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu);
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
#[derive(Component)]
struct DifficultySettingButton;

#[derive(Component)]
struct DifficultySettingButtonText;

fn setup_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    difficulty: Res<DifficultySetting>,
) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(45.0),
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
        font_size: 30.0,
        color: TITLE_TEXT,
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
            TextBundle::from_section("UNDEFENDED!", title_text_style).with_style(Style {
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
            parent.spawn(TextBundle::from_section("PLAY", button_text_style.clone()));
        })
        .id();

    let audio_settings_title = commands
        .spawn(
            TextBundle::from_section("- AUDIO -", subtitle_text_style.clone()).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let difficulty_title = commands
        .spawn(
            TextBundle::from_section("- DIFFICULTY -", subtitle_text_style).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let difficulty_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Difficulty,
            SfxSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("{}", *difficulty), button_text_style.clone()),
                DifficultySettingButtonText,
            ));
        })
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
                TextBundle::from_section(format!("MUSIC {}%", **music), button_text_style),
                MusicSettingButtonText,
            ));
        })
        .id();

    commands.entity(container).push_children(&[
        title,
        play_button,
        difficulty_title,
        difficulty_button,
        audio_settings_title,
        sfx_button,
        music_button,
    ]);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(35.),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    width: Val::Percent(100.),
                    column_gap: Val::Px(10.),
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..default()
            },
            MainMenuMarker,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    " \nJUMP\nINTERACT\nMOVE",
                    TextStyle {
                        font: fonts.main.clone(),
                        font_size: 20.0,
                        color: UI_TEXT,
                    },
                )
                .with_alignment(TextAlignment::Right),
                ..Default::default()
            });
            parent.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection {
                        value: "PAD\n".to_string(),
                        style: TextStyle {
                            font: fonts.main.clone(),
                            font_size: 20.0,
                            color: UI_TEXT,
                        },
                    },
                    TextSection {
                        value: "SOUTH\nWEST\nL STICK".to_string(),
                        style: TextStyle {
                            font: fonts.main.clone(),
                            font_size: 20.0,
                            color: ALT_TEXT,
                        },
                    },
                ]),
                ..Default::default()
            });
            parent.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection {
                        value: "BOARD\n".to_string(),
                        style: TextStyle {
                            font: fonts.main.clone(),
                            font_size: 20.0,
                            color: UI_TEXT,
                        },
                    },
                    TextSection {
                        value: "SPACE\nR\nWASD OR ARROWS".to_string(),
                        style: TextStyle {
                            font: fonts.main.clone(),
                            font_size: 20.0,
                            color: ALT_TEXT,
                        },
                    },
                ]),
                ..Default::default()
            });
        });
}

#[derive(Component)]
enum MenuButton {
    Play,
    Sfx,
    Music,
    Difficulty,
}

fn button_actions(
    buttons: Query<&MenuButton>,
    mut events: EventReader<NavEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut music_setting: ResMut<MusicSetting>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<SfxSettingButtonText>>,
        Query<&mut Text, With<MusicSettingButtonText>>,
        Query<&mut Text, With<DifficultySettingButtonText>>,
    )>,
    mut sfx_setting: ResMut<SfxSetting>,
    mut difficulty_setting: ResMut<DifficultySetting>,
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
                    text.sections[0].value = format!("MUSIC {}%", **music_setting);
                }
            }
            MenuButton::Difficulty => {
                *difficulty_setting = difficulty_setting.next();

                for mut text in text_queries.p2().iter_mut() {
                    text.sections[0].value = format!("{}", *difficulty_setting);
                }
            }
        }
    }
}

fn sfx_volume(mut commands: Commands, sfx_setting: Res<SfxSetting>, game_audio: Res<Sounds>) {
    // Do not run when SfxSetting is first added by SavePlugin
    if !sfx_setting.is_changed() || sfx_setting.is_added() {
        return;
    }

    commands.spawn(AudioBundle {
        source: game_audio.build.clone(),
        settings: PlaybackSettings::ONCE.with_volume(**sfx_setting as f32 / 100.),
    });
}

fn music_volume(
    music_setting: Res<MusicSetting>,
    music_query: Query<&AudioSink, With<MusicController>>,
) {
    // Do not run when MusicSetting is first added by SavePlugin
    if !music_setting.is_changed() || music_setting.is_added() {
        return;
    }

    if let Ok(sink) = music_query.get_single() {
        sink.set_volume(**music_setting as f32 / 100.)
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
