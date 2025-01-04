use bevy::{
    audio::{AudioSink, Volume},
    prelude::*,
};
use bevy_alt_ui_navigation_lite::prelude::*;

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
    let button_style = Node {
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
    let title_text_style = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 50.0,
            ..default()
        },
        TextColor(TITLE_TEXT.into()),
    );
    let subtitle_text_style = (
        TextFont {
            font: fonts.main.clone(),
            font_size: 25.0,
            ..default()
        },
        TextColor(TITLE_TEXT.into()),
    );

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
            MainMenuMarker,
        ))
        .id();

    let title = commands
        .spawn((
            Text::new("UNDEFENDED!"),
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

    let play_button = commands
        .spawn((
            Button,
            button_style.clone(),
            BackgroundColor(NORMAL_BUTTON.into()),
            Focusable::default(),
            MenuButton::Play,
            PlayButton,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("PLAY"), button_text_style.clone()));
        })
        .id();

    let audio_settings_title = commands
        .spawn((
            Text::new("- AUDIO -"),
            subtitle_text_style.clone(),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let difficulty_title = commands
        .spawn((
            Text::new("- DIFFICULTY -"),
            subtitle_text_style,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let difficulty_button = commands
        .spawn((
            Button,
            button_style.clone(),
            BackgroundColor(NORMAL_BUTTON.into()),
            Focusable::default(),
            MenuButton::Difficulty,
            DifficultySettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{}", *difficulty)),
                button_text_style.clone(),
                DifficultySettingButtonText,
            ));
        })
        .id();

    let sfx_button = commands
        .spawn((
            Button,
            button_style.clone(),
            BackgroundColor(NORMAL_BUTTON.into()),
            Focusable::default(),
            MenuButton::Sfx,
            SfxSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("SFX {}%", **sfx)),
                button_text_style.clone(),
                SfxSettingButtonText,
            ));
        })
        .id();

    let music_button = commands
        .spawn((
            Button,
            button_style,
            BackgroundColor(NORMAL_BUTTON.into()),
            Focusable::default(),
            MenuButton::Music,
            MusicSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("MUSIC {}%", **music)),
                button_text_style,
                MusicSettingButtonText,
            ));
        })
        .id();

    commands.entity(container).add_children(&[
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
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                width: Val::Percent(100.),
                column_gap: Val::Px(10.),
                justify_content: JustifyContent::Center,
                ..default()
            },
            MainMenuMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("JUMP\nINTERACT\nMOVE"),
                TextFont {
                    font: fonts.main.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(UI_TEXT.into()),
                TextLayout::new_with_justify(JustifyText::Right),
            ));
            parent.spawn((
                Text::new("\u{21A7}\n\u{21A4}\n\u{21CB}\u{21CE}"),
                TextFont {
                    font: fonts.prompts.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(ALT_TEXT.into()),
            ));
            parent.spawn((
                Text::new(concat!(
                    "SPACE\n",
                    "\u{FF32}\n",
                    "\u{FF37}\u{FF21}\u{FF33}\u{FF24}\u{23F6}\u{23F4}\u{23F5}\u{23F7}"
                )),
                TextFont {
                    font: fonts.prompts.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(ALT_TEXT.into()),
            ));
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
                    text.0 = format!("SFX {}%", **sfx_setting);
                }
            }
            MenuButton::Music => {
                if **music_setting == 0 {
                    **music_setting = 100;
                } else {
                    **music_setting -= 10;
                }

                for mut text in text_queries.p1().iter_mut() {
                    text.0 = format!("MUSIC {}%", **music_setting);
                }
            }
            MenuButton::Difficulty => {
                *difficulty_setting = difficulty_setting.next();

                for mut text in text_queries.p2().iter_mut() {
                    text.0 = format!("{}", *difficulty_setting);
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

    commands.spawn((
        AudioPlayer(game_audio.build.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::new(**sfx_setting as f32 / 100.)),
    ));
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
