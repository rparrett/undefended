#![allow(clippy::too_many_arguments, clippy::type_complexity)]

#[cfg(feature = "debugdump")]
use std::{fs::File, io::Write};

use bevy::{
    asset::AssetMetaCheck, audio::Volume, pbr::CascadeShadowConfigBuilder, prelude::*,
    transform::TransformSystem,
};
use bevy_alt_ui_navigation_lite::{systems::InputMapping, DefaultNavigationPlugins};
use bevy_dolly::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::HookPlugin;
use bevy_tnua::prelude::*;
use bevy_tnua_rapier3d::{TnuaRapier3dIOBundle, TnuaRapier3dPlugin, TnuaRapier3dSensorShape};
use bevy_two_entities::tuple::{TupleQueryExt, TupleQueryMutExt};
use game_over::GameOverPlugin;
use leafwing_input_manager::prelude::*;

use enemy::{Enemy, EnemyPlugin};
use loading::{LoadingPlugin, Models, Sounds};
use main_menu::MainMenuPlugin;
use map::{
    map_to_world, Floor, Item, ItemSpawner, Lava, MapPlugin, MovingFloor, PlacedTower, TilePos,
    START_TILE,
};
use outline::OutlinePlugin;
use save::SavePlugin;
use settings::{MusicSetting, SfxSetting};
use starfield::StarfieldPlugin;
use tower::{Ammo, SpawnTowerEvent, Tower, TowerPlugin};
use ui::UiPlugin;
use waves::{WavePlugin, WaveState, Waves};

mod enemy;
mod game_over;
mod loading;
mod main_menu;
mod map;
mod outline;
mod save;
mod settings;
mod starfield;
mod tower;
mod ui;
mod waves;

#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Run,
    Jump,
    Grab,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct TileProbe;

#[derive(Component)]
struct ItemProbe;

#[derive(Component)]
struct LastTile(UVec2);

#[derive(Component, Reflect)]
struct SelectedTile(Option<Entity>);

#[derive(Component, Default, Reflect)]
struct SelectedItem(Option<Entity>);

#[derive(Component)]
struct GrabbedItem;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct Lives(u32);
impl Default for Lives {
    fn default() -> Self {
        Self(3)
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Pipelines,
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct AfterPhysics;

#[derive(Component)]
struct MusicController;

#[derive(Event)]
struct SpawnPlayerEvent(UVec2);

#[derive(Resource, Default)]
struct Won(bool);

#[derive(Component)]
struct Persist;

const CAMERA_OFFSET: Vec3 = Vec3::new(0., 10., 6.);

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "UNDEFENDED!".to_string(),
                    resizable: false,
                    canvas: Some("#bevy".to_string()),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // Workaround for Bevy attempting to load .meta files in wasm builds. On itch,
                // the CDN serves HTTP 403 errors instead of 404 when files don't exist, which
                // causes Bevy to break.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    );

    app.init_state::<GameState>()
        .add_event::<SpawnPlayerEvent>();

    // TODO we may need apply_deferred somewhere in here
    app.configure_sets(
        PostUpdate,
        AfterPhysics
            .after(PhysicsSet::Writeback)
            .before(TransformSystem::TransformPropagate),
    );

    app.add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                cursor,
                item_probe,
                spawn_player,
                track_last_tile,
                lava,
                grab,
                build_tower,
                feed_tower,
                game_over,
            )
                .distributive_run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, apply_controls.in_set(TnuaUserControlsSystemSet))
        .add_systems(
            PostUpdate,
            update_camera
                .in_set(AfterPhysics)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            reset_item_on_grab
                .in_set(AfterPhysics)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Loading), setup_camera)
        .add_systems(OnExit(GameState::Pipelines), start_music)
        .add_systems(OnExit(GameState::GameOver), reset);

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(TnuaControllerPlugin::default())
        .add_plugins(TnuaRapier3dPlugin::default())
        .add_systems(
            PostUpdate,
            Dolly::<MainCamera>::update_active
                .in_set(AfterPhysics)
                .in_set(DollyUpdateSet)
                .after(update_camera),
        )
        .add_plugins(InputManagerPlugin::<Action>::default())
        .insert_resource(InputMapping {
            keyboard_navigation: true,
            ..default()
        })
        .add_plugins(DefaultNavigationPlugins)
        .add_plugins(HookPlugin);

    app.init_resource::<Lives>()
        .register_type::<Lives>()
        .init_resource::<Won>()
        .add_plugins(LoadingPlugin)
        .add_plugins(StarfieldPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(TowerPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(SavePlugin)
        .add_plugins(UiPlugin)
        .add_plugins(WavePlugin)
        .add_plugins(GameOverPlugin)
        .add_plugins(OutlinePlugin);

    #[cfg(feature = "inspector")]
    {
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.register_type::<SelectedTile>();
        app.register_type::<SelectedItem>();
    }

    #[cfg(feature = "debugdump")]
    {
        let settings = bevy_mod_debugdump::schedule_graph::Settings {
            ambiguity_enable: false,
            ambiguity_enable_on_world: false,
            ..Default::default()
        };

        let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &settings);
        let mut f = File::create("debugdump_update.dot").unwrap();
        f.write_all(dot.as_bytes()).unwrap();

        let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, PostUpdate, &settings);
        let mut f = File::create("debugdump_postupdate.dot").unwrap();
        f.write_all(dot.as_bytes()).unwrap();

        return;
    }

    #[cfg(not(feature = "debugdump"))]
    app.run();
}

fn setup(mut commands: Commands, mut spawn_player_events: EventWriter<SpawnPlayerEvent>) {
    spawn_player_events.send(SpawnPlayerEvent(START_TILE));

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -1.0, -1.0, -1.0)),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 30.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Rig::builder()
            .with(Position::new(Vec3::ZERO))
            .with(Smooth::new_position(0.25))
            .with(Arm::new(CAMERA_OFFSET))
            .with(Smooth::new_position(0.25))
            .with(LookAt::new(Vec3::ZERO + Vec3::Y).tracking_smoothness(0.25))
            .build(),
        Camera3dBundle {
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        Persist,
    ));
}

fn apply_controls(
    action_state_query: Query<&ActionState<Action>, With<Player>>,
    mut query: Query<&mut TnuaController>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let mut turn_in_place = false;

    if action_state.pressed(&Action::Run) {
        let axis_pair = action_state.clamped_axis_pair(&Action::Run).unwrap();

        let vec = Vec3::new(axis_pair.x(), 0., -axis_pair.y());
        turn_in_place = vec.x.abs() < 0.3 && vec.z.abs() < 0.3;

        direction += vec;
    }

    let normalized = direction.normalize_or_zero();
    let with_speed = normalized * 4.3;

    let jump = action_state.pressed(&Action::Jump);

    for mut controls in query.iter_mut() {
        controls.basis(TnuaBuiltinWalk {
            desired_velocity: if turn_in_place {
                Vec3::ZERO
            } else {
                with_speed
            },
            desired_forward: normalized,
            float_height: 1.0,
            cling_distance: 0.5,
            acceleration: 50.0,
            air_acceleration: 10.0,
            turning_angvel: 5.0,
            ..default()
        });

        if jump {
            controls.action(TnuaBuiltinJump {
                height: 2.0,
                shorten_extra_gravity: 40.0,
                ..default()
            });
        }
    }
}

fn update_camera(player_query: Query<&Transform, With<Player>>, mut rig_query: Query<&mut Rig>) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    let Ok(mut rig) = rig_query.get_single_mut() else {
        return;
    };

    rig.driver_mut::<Position>().position = player.translation;
    // rig.driver_mut::<Rotation>().rotation = player.rotation;
    rig.driver_mut::<LookAt>().target = player.translation + Vec3::Y;
}

fn cursor(
    mut collision_events: EventReader<CollisionEvent>,
    cursor_query: Query<&Parent, With<Cursor>>,
    floor_query: Query<Entity, With<Floor>>,
    mut selected_tile_query: Query<&mut SelectedTile>,
) {
    for evt in collision_events.read() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let queries = (&cursor_query, &floor_query);
                let Some((cursor, floor_entity)) = queries.get_both(*e1, *e2) else {
                    continue;
                };

                if let Ok(mut selected_tile) = selected_tile_query.get_mut(cursor.get()) {
                    selected_tile.0 = Some(floor_entity);
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let queries = (&cursor_query, &floor_query);
                let Some((cursor, _floor_entity)) = queries.get_both(*e1, *e2) else {
                    continue;
                };

                if let Ok(mut selected_tile) = selected_tile_query.get_mut(cursor.get()) {
                    selected_tile.0 = None;
                }
            }
        }
    }
}

fn item_probe(
    mut collision_events: EventReader<CollisionEvent>,
    probe_query: Query<&Parent, With<ItemProbe>>,
    item_query: Query<Entity, With<Item>>,
    mut selected_item_query: Query<&mut SelectedItem>,
) {
    for evt in collision_events.read() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let queries = (&probe_query, &item_query);
                let Some((probe_entity, item_entity)) = queries.get_both(*e1, *e2) else {
                    continue;
                };

                if let Ok(mut selected_item) = selected_item_query.get_mut(probe_entity.get()) {
                    selected_item.0 = Some(item_entity);
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let queries = (&probe_query, &item_query);
                if !queries.both(*e1, *e2) {
                    continue;
                };

                for mut selected_item in selected_item_query.iter_mut() {
                    selected_item.0 = None;
                }
            }
        }
    }
}

fn track_last_tile(
    mut collision_events: EventReader<CollisionEvent>,
    probe_query: Query<&Parent, With<TileProbe>>,
    floor_query: Query<&TilePos, (With<Floor>, Without<MovingFloor>)>,
    mut last_tile_query: Query<&mut LastTile>,
) {
    for evt in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = evt {
            let queries = (&probe_query, &floor_query);
            let Some((probe_entity, tile_pos)) = queries.get_both(*e1, *e2) else {
                continue;
            };

            if let Ok(mut last_tile) = last_tile_query.get_mut(probe_entity.get()) {
                last_tile.0 = tile_pos.0;
            }
        }
    }
}

fn lava(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    lava_query: Query<&Lava>,
    mut player_query: Query<(&LastTile, &Children, &mut Transform), With<Player>>,
    item_query: Query<Entity, With<Item>>,
    tower_query: Query<&TilePos, With<Tower>>,
) {
    for evt in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = evt {
            let mut queries = (&lava_query, &mut player_query);
            let Some((_, (last_tile, children, mut transform))) = queries.get_both_mut(*e1, *e2)
            else {
                continue;
            };

            let pos = if tower_query.iter().any(|pos| pos.0 == last_tile.0) {
                START_TILE
            } else {
                last_tile.0
            };

            transform.translation = map_to_world(pos);

            for item_entity in item_query.iter_many(children) {
                commands.entity(item_entity).despawn_recursive();
            }
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut events: EventReader<SpawnPlayerEvent>,
    models: Res<Models>,
) {
    for event in events.read() {
        commands
            .spawn((
                Player,
                Name::new("Player"),
                SpatialBundle {
                    transform: Transform::from_translation(map_to_world(event.0) + Vec3::Y * 0.5),
                    ..default()
                },
                RigidBody::Dynamic,
                Velocity::default(),
                Collider::capsule_y(0.30, 0.5),
                ActiveEvents::COLLISION_EVENTS,
                ExternalForce::default(),
                ReadMassProperties::default(),
                LastTile(event.0),
                SelectedTile(None),
                SelectedItem(None),
                InputManagerBundle::<Action> {
                    action_state: ActionState::default(),
                    input_map: InputMap::default()
                        .insert(Action::Jump, KeyCode::Space)
                        .insert(Action::Jump, GamepadButtonType::South)
                        .insert(Action::Grab, KeyCode::KeyR)
                        .insert(Action::Grab, GamepadButtonType::West)
                        .insert(Action::Run, DualAxis::left_stick())
                        .insert(
                            Action::Run,
                            VirtualDPad {
                                up: KeyCode::KeyW.into(),
                                down: KeyCode::KeyS.into(),
                                left: KeyCode::KeyA.into(),
                                right: KeyCode::KeyD.into(),
                            },
                        )
                        .insert(
                            Action::Run,
                            VirtualDPad {
                                up: KeyCode::ArrowUp.into(),
                                down: KeyCode::ArrowDown.into(),
                                left: KeyCode::ArrowLeft.into(),
                                right: KeyCode::ArrowRight.into(),
                            },
                        )
                        .build(),
                },
            ))
            .insert((
                TnuaRapier3dIOBundle::default(),
                TnuaControllerBundle::default(),
                TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.49)),
            ))
            .with_children(|parent| {
                parent.spawn(
                    SceneRoot(models.player.clone()),
                    Transform::from_xyz(0., -0.4, 0.),
                );

                // probe for current tile
                parent.spawn((
                    TileProbe,
                    Name::new("TileProbe"),
                    SpatialBundle::default(),
                    Collider::segment(Vec3::new(0., 0., 0.), Vec3::new(0.0, -2.0, 0.)),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));

                // probe for current tile
                parent.spawn((
                    ItemProbe,
                    Name::new("ItemProbe"),
                    SpatialBundle::default(),
                    Collider::segment(Vec3::new(0., -0.25, 0.), Vec3::new(0.0, -0.25, -1.0)),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));

                // cursor
                parent.spawn((
                    Cursor,
                    Name::new("Cursor"),
                    SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.9, -1.5)),
                    Collider::segment(Vec3::new(0.0, 0., 0.), Vec3::new(0.0, -2.1, 0.)),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));
            });
    }
}

fn grab(
    mut commands: Commands,
    player_query: Query<(Entity, &Children, &ActionState<Action>, &SelectedItem), With<Player>>,
    grabbed_item_query: Query<(), With<GrabbedItem>>,
    mut item_query: Query<&Item>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
) {
    let Ok((entity, children, action_state, selected_item)) = player_query.get_single() else {
        return;
    };

    if !action_state.just_pressed(&Action::Grab) {
        return;
    }

    let Some(selected_item) = selected_item.0 else {
        return;
    };
    if item_query.get_mut(selected_item).is_err() {
        return;
    };

    // player is already holding an item
    if grabbed_item_query.iter_many(children).next().is_some() {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });

        return;
    }

    commands
        .entity(selected_item)
        .set_parent(entity)
        .remove::<Collider>()
        .insert(GrabbedItem);
}

fn build_tower(
    mut commands: Commands,
    player_query: Query<(&Children, &ActionState<Action>, &SelectedTile), With<Player>>,
    grabbed_item_query: Query<(Entity, &Item)>,
    invalid_tile_query: Query<(), Or<(With<MovingFloor>, With<PlacedTower>, With<ItemSpawner>)>>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
    mut events: EventWriter<SpawnTowerEvent>,
) {
    let Ok((children, action_state, selected_tile)) = player_query.get_single() else {
        return;
    };

    if !action_state.just_pressed(&Action::Grab) {
        return;
    }

    let Some((entity, item)) = grabbed_item_query.iter_many(children).next() else {
        return;
    };

    if *item != Item::TowerKit {
        return;
    }

    let Some(selected_tile) = selected_tile.0 else {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });
        return;
    };

    let invalid = invalid_tile_query.get(selected_tile).is_ok();
    if invalid {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });
        return;
    }

    commands.entity(entity).despawn_recursive();

    events.send(SpawnTowerEvent(selected_tile));
}

fn feed_tower(
    mut commands: Commands,
    player_query: Query<(&Children, &ActionState<Action>, &SelectedTile), With<Player>>,
    grabbed_item_query: Query<(Entity, &Item)>,
    placed_tower_query: Query<&PlacedTower>,
    mut tower_query: Query<&mut Ammo, With<Tower>>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
) {
    let Ok((children, action_state, selected_tile)) = player_query.get_single() else {
        return;
    };

    if !action_state.just_pressed(&Action::Grab) {
        return;
    }

    let Some((entity, item)) = grabbed_item_query.iter_many(children).next() else {
        return;
    };

    if *item != Item::LaserAmmo {
        return;
    }

    let Some(selected_tile) = selected_tile.0 else {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });

        return;
    };

    let Ok(placed_tower) = placed_tower_query.get(selected_tile) else {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });
        return;
    };

    let Ok(mut ammo) = tower_query.get_mut(placed_tower.0) else {
        commands.spawn(AudioBundle {
            source: game_audio.bad.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new(**audio_setting as f32 / 100.)),
        });
        return;
    };

    ammo.current = ammo.max;

    commands.spawn(AudioBundle {
        source: game_audio.feed.clone(),
        settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(**audio_setting as f32 / 100.)),
    });

    commands.entity(entity).despawn_recursive();
}

fn reset_item_on_grab(mut item_query: Query<&mut Transform, Added<GrabbedItem>>) {
    for mut transform in item_query.iter_mut() {
        transform.translation = Vec3::new(0., -0.4, -0.75);
        transform.rotation = Quat::IDENTITY;
    }
}

fn start_music(
    mut commands: Commands,
    music_setting: Res<MusicSetting>,
    audio_assets: Res<Sounds>,
) {
    commands.spawn((
        AudioBundle {
            source: audio_assets.music.clone(),
            settings: PlaybackSettings::LOOP
                .with_volume(Volume::new(**music_setting as f32 / 100.)),
        },
        MusicController,
        Persist,
    ));
}

fn game_over(
    mut commands: Commands,
    lives: Res<Lives>,
    waves: Res<Waves>,
    wave_state: Res<WaveState>,
    enemies: Query<(), With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if waves.current().is_none() && wave_state.remaining == 0 && enemies.iter().len() == 0 {
        commands.insert_resource(Won(true));
        next_state.set(GameState::GameOver);
    }

    if lives.0 == 0 {
        commands.insert_resource(Won(false));
        next_state.set(GameState::GameOver);
    }
}

fn reset(
    mut commands: Commands,
    roots_query: Query<
        Entity,
        (
            Without<Window>,
            Without<Persist>,
            With<Children>,
            Without<Parent>,
        ),
    >,
    orphans_query: Query<
        Entity,
        (
            Without<Window>,
            Without<Persist>,
            Without<Children>,
            Without<Parent>,
        ),
    >,
) {
    for entity in roots_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in orphans_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.insert_resource(Lives::default());
}
