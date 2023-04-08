use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_dolly::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
    TnuaPlatformerPlugin, TnuaRapier3dPlugin, TnuaRapier3dSensorShape,
};
use bevy_ui_navigation::{systems::InputMapping, DefaultNavigationPlugins};
use leafwing_input_manager::prelude::*;

use enemy::EnemyPlugin;
use loading::{LoadingPlugin, Models, Sounds};
use main_menu::MainMenuPlugin;
use map::{map_to_world, Floor, Item, Lava, MapPlugin, MovingFloor, TilePos, START_TILE};
use save::SavePlugin;
use settings::MusicSetting;
use starfield::StarfieldPlugin;
use tower::{SpawnTowerEvent, TowerPlugin};

mod enemy;
mod loading;
mod main_menu;
mod map;
mod save;
mod settings;
mod starfield;
mod tower;
mod ui;

#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
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

#[derive(Component, Default, Reflect)]
struct SelectedTile(Option<UVec2>);

#[derive(Component, Default, Reflect)]
struct SelectedItem(Option<Entity>);

#[derive(Component)]
struct GrabbedItem;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub struct AfterPhysics;

#[derive(Resource)]
struct MusicController(Handle<AudioSink>);

struct SpawnPlayerEvent(UVec2);

const CAMERA_OFFSET: Vec3 = Vec3::new(0., 10., 6.);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "TODO".to_string(),
            decorations: false,
            ..default()
        }),
        ..default()
    }));

    #[cfg(feature = "inspector")]
    {
        app.add_plugin(WorldInspectorPlugin::new());
        app.add_plugin(RapierDebugRenderPlugin::default());
        app.register_type::<SelectedTile>();
        app.register_type::<SelectedItem>();
    }

    app.add_state::<GameState>().add_event::<SpawnPlayerEvent>();

    app.configure_set(
        AfterPhysics
            .after(PhysicsSet::Writeback)
            .before(CoreSet::PostUpdate),
    );

    app.add_system(setup.in_schedule(OnEnter(GameState::Playing)))
        .add_system(apply_controls.in_set(OnUpdate(GameState::Playing)))
        .add_system(update_camera.in_set(OnUpdate(GameState::Playing)))
        .add_system(cursor.in_set(OnUpdate(GameState::Playing)))
        .add_system(item_probe.in_set(OnUpdate(GameState::Playing)))
        .add_system(spawn_player.in_set(OnUpdate(GameState::Playing)))
        .add_system(track_last_tile.in_set(OnUpdate(GameState::Playing)))
        .add_system(lava.in_set(OnUpdate(GameState::Playing)))
        .add_system(grab.in_set(OnUpdate(GameState::Playing)))
        .add_system(build_tower.in_set(OnUpdate(GameState::Playing)))
        .add_system(
            reset_item_on_grab
                .in_base_set(AfterPhysics)
                .run_if(in_state(GameState::Playing)),
        )
        .add_system(setup_camera.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(start_music.in_schedule(OnEnter(GameState::MainMenu)));

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(TnuaPlatformerPlugin)
        .add_plugin(TnuaRapier3dPlugin)
        .add_system(Dolly::<MainCamera>::update_active)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .insert_resource(InputMapping {
            keyboard_navigation: true,
            ..default()
        })
        .add_plugins(DefaultNavigationPlugins);

    app.add_plugin(LoadingPlugin)
        .add_plugin(StarfieldPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(TowerPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SavePlugin)
        .run();
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
        ..default()
    });
}

fn setup_camera(mut commands: Commands) {
    // camera
    commands.spawn((
        MainCamera,
        Rig::builder()
            .with(Position::new(Vec3::ZERO))
            //.with(Rotation::new(Quat::IDENTITY))
            //.with(Smooth::new_position(1.25).predictive(true))
            .with(Smooth::new_position(0.25))
            .with(Arm::new(CAMERA_OFFSET))
            .with(Smooth::new_position(0.25))
            .with(
                LookAt::new(Vec3::ZERO + Vec3::Y).tracking_smoothness(0.25), //.tracking_predictive(true),
            )
            .build(),
        Camera3dBundle {
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
    ));
}

fn apply_controls(
    action_state_query: Query<&ActionState<Action>, With<Player>>,
    mut query: Query<&mut TnuaPlatformerControls>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let mut turn_in_place = false;

    if action_state.pressed(Action::Run) {
        let axis_pair = action_state.clamped_axis_pair(Action::Run).unwrap();

        let vec = Vec3::new(axis_pair.x(), 0., -axis_pair.y());
        turn_in_place = vec.x.abs() < 0.3 && vec.z.abs() < 0.3;

        direction += vec;
    }

    let jump = action_state.pressed(Action::Jump);

    for mut controls in query.iter_mut() {
        *controls = TnuaPlatformerControls {
            desired_velocity: if turn_in_place { Vec3::ZERO } else { direction },
            desired_forward: direction.normalize(),
            jump: jump.then_some(1.0),
        };
    }
}

fn update_camera(player_query: Query<&Transform, With<Player>>, mut rig_query: Query<&mut Rig>) {
    let count = player_query.iter().len();

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
    cursor_query: Query<Entity, With<Cursor>>,
    floor_query: Query<(Entity, &TilePos), With<Floor>>,
    mut selected_tile_query: Query<&mut SelectedTile>,
) {
    for evt in collision_events.iter() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let is_cursor = cursor_query.iter_many([e1, e2]).count() > 0;
                let is_floor = floor_query.iter_many([e1, e2]).count() > 0;

                if is_cursor && is_floor {
                    for (_, tile_pos) in floor_query.iter_many([e1, e2]) {
                        for mut selected_tile in selected_tile_query.iter_mut() {
                            selected_tile.0 = Some(tile_pos.0);
                        }
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let is_cursor = cursor_query.iter_many([e1, e2]).count() > 0;
                let is_floor = floor_query.iter_many([e1, e2]).count() > 0;

                if is_cursor && is_floor {
                    for mut selected_tile in selected_tile_query.iter_mut() {
                        selected_tile.0 = None;
                    }
                }
            }
        }
    }
}

fn item_probe(
    mut collision_events: EventReader<CollisionEvent>,
    probe_query: Query<Entity, With<ItemProbe>>,
    item_query: Query<Entity, With<Item>>,
    mut selected_item_query: Query<&mut SelectedItem>,
) {
    for evt in collision_events.iter() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let is_probe = probe_query.iter_many([e1, e2]).count() > 0;
                let is_item = item_query.iter_many([e1, e2]).count() > 0;

                if is_probe && is_item {
                    for entity in item_query.iter_many([e1, e2]) {
                        for mut selected_item in selected_item_query.iter_mut() {
                            selected_item.0 = Some(entity);
                            info!("selected_item: {:?}", selected_item.0);
                        }
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let is_probe = probe_query.iter_many([e1, e2]).count() > 0;
                let is_item = item_query.iter_many([e1, e2]).count() > 0;

                if is_probe && is_item {
                    for mut selected_item in selected_item_query.iter_mut() {
                        selected_item.0 = None;
                        info!("selected_item: {:?}", selected_item.0);
                    }
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
    for evt in collision_events.iter() {
        if let CollisionEvent::Started(e1, e2, _) = evt {
            let is_probe = probe_query.iter_many([e1, e2]).count() > 0;
            let is_floor = floor_query.iter_many([e1, e2]).count() > 0;

            if is_probe && is_floor {
                for parent in probe_query.iter_many([e1, e2]) {
                    if let Ok(mut last_tile) = last_tile_query.get_mut(**parent) {
                        for tile_pos in floor_query.iter_many([e1, e2]) {
                            last_tile.0 = tile_pos.0;
                        }
                    }
                }
            }
        }
    }
}

fn lava(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    lava_query: Query<&Lava>,
    player_query: Query<&LastTile, With<Player>>,
    mut spawn_player_events: EventWriter<SpawnPlayerEvent>,
) {
    for evt in collision_events.iter() {
        if let CollisionEvent::Started(e1, e2, _) = evt {
            let is_lava = lava_query.iter_many([e1, e2]).count() > 0;
            let is_player = player_query.iter_many([e1, e2]).count() > 0;

            if is_lava && is_player {
                if let Ok(last_tile) = player_query.get(*e1) {
                    commands.entity(*e1).despawn_recursive();
                    spawn_player_events.send(SpawnPlayerEvent(last_tile.0));
                }
                if let Ok(last_tile) = player_query.get(*e2) {
                    commands.entity(*e2).despawn_recursive();
                    spawn_player_events.send(SpawnPlayerEvent(last_tile.0));
                }
            }
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut events: EventReader<SpawnPlayerEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    models: Res<Models>,
) {
    for event in events.iter() {
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
                LastTile(event.0),
                SelectedTile(None),
                SelectedItem(None),
                TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.49)),
                TnuaPlatformerBundle::new_with_config(TnuaPlatformerConfig {
                    full_speed: 4.0,
                    full_jump_height: 2.0,
                    up: Vec3::Y,
                    forward: -Vec3::Z,
                    float_height: 1.0,
                    cling_distance: 0.5,
                    spring_strengh: 400.0,
                    spring_dampening: 1.2,
                    acceleration: 50.0,
                    air_acceleration: 10.0,
                    coyote_time: 0.15,
                    jump_start_extra_gravity: 30.0,
                    jump_fall_extra_gravity: 20.0,
                    jump_shorten_extra_gravity: 40.0,
                    free_fall_behavior: TnuaFreeFallBehavior::LikeJumpShorten,
                    tilt_offset_angvel: 5.0,
                    tilt_offset_angacl: 500.0,
                    turning_angvel: 5.0,
                }),
                InputManagerBundle::<Action> {
                    action_state: ActionState::default(),
                    input_map: InputMap::default()
                        .insert(KeyCode::Space, Action::Jump)
                        .insert(GamepadButtonType::South, Action::Jump)
                        .insert(KeyCode::R, Action::Grab)
                        .insert(GamepadButtonType::West, Action::Grab)
                        .insert(DualAxis::left_stick(), Action::Run)
                        .insert(
                            VirtualDPad {
                                up: QwertyScanCode::W.into(),
                                down: QwertyScanCode::S.into(),
                                left: QwertyScanCode::A.into(),
                                right: QwertyScanCode::D.into(),
                            },
                            Action::Run,
                        )
                        .insert(
                            VirtualDPad {
                                up: QwertyScanCode::Up.into(),
                                down: QwertyScanCode::Down.into(),
                                left: QwertyScanCode::Left.into(),
                                right: QwertyScanCode::Right.into(),
                            },
                            Action::Run,
                        )
                        .build(),
                },
            ))
            .with_children(|parent| {
                // parent.spawn(PbrBundle {
                //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                //     transform: Transform::from_xyz(0.0, -0.5, 0.0),
                //     ..default()
                // });
                parent.spawn(SceneBundle {
                    scene: models.player.clone(),
                    transform: Transform::from_xyz(0., -0.4, 0.),
                    ..default()
                });

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
                    Collider::segment(Vec3::new(0., 0.0, 0.), Vec3::new(0.0, 0.0, -1.0)),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));

                // cursor
                parent.spawn((
                    Cursor,
                    Name::new("Cursor"),
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                        material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
                        transform: Transform::from_xyz(0.0, -0.9, -1.5),
                        ..default()
                    },
                    Collider::segment(Vec3::new(0.0, 0., 0.), Vec3::new(0.0, -1.0, 0.)),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));
            });
    }
}

fn grab(
    mut commands: Commands,
    player_query: Query<(Entity, &ActionState<Action>), With<Player>>,
    selected_item_query: Query<&SelectedItem>,
    mut item_query: Query<&Item>,
) {
    let Ok((entity, action_state)) = player_query.get_single() else {
        return;
    };

    if !action_state.just_pressed(Action::Grab) {
        return;
    }

    let Ok(selected_item) = selected_item_query.get_single() else {
        return
    };
    let Some(selected_item) = selected_item.0 else {
        return
    };
    if item_query.get_mut(selected_item).is_err() {
        return;
    };

    commands
        .entity(selected_item)
        .set_parent(entity)
        .remove::<Collider>()
        .insert(GrabbedItem);
}

fn build_tower(
    mut commands: Commands,
    player_query: Query<(Entity, &Children, &ActionState<Action>), With<Player>>,
    selected_tile_query: Query<&SelectedTile>,
    grabbed_item_query: Query<(Entity, &Item)>,
    mut events: EventWriter<SpawnTowerEvent>,
) {
    let Ok((entity, children, action_state)) = player_query.get_single() else {
        return;
    };

    if !action_state.just_pressed(Action::Grab) {
        return;
    }

    let Ok(selected_tile) = selected_tile_query.get_single() else {
        return
    };

    let Some(selected_tile) = selected_tile.0 else {
        return
    };

    for (entity, item) in grabbed_item_query.iter_many(children) {
        if *item != Item::TowerKit {
            continue;
        }

        commands.entity(entity).despawn_recursive();
        events.send(SpawnTowerEvent(selected_tile))
    }
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
    audio_sinks: Res<Assets<AudioSink>>,
    audio: Res<Audio>,
) {
    let handle = audio_sinks.get_handle(audio.play_with_settings(
        audio_assets.music.clone(),
        PlaybackSettings::LOOP.with_volume(**music_setting as f32 / 100.),
    ));
    commands.insert_resource(MusicController(handle));
}
