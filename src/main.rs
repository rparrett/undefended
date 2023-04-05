//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_dolly::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig,
    TnuaPlatformerControls, TnuaPlatformerPlugin, TnuaRapier3dPlugin,
};
use loading::LoadingPlugin;
use map::{map_to_world, Floor, FloorMaterials, MapPlugin};
use starfield::StarfieldPlugin;

mod loading;
mod map;
mod starfield;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Cursor;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

const CAMERA_OFFSET: Vec3 = Vec3::new(0., 10., 6.);
const START_TILE: UVec2 = UVec2::new(10, 9);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_startup_system(spawn_player)
        .add_system(apply_controls)
        .add_system(update_camera)
        .add_system(cursor)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(TnuaPlatformerPlugin)
        .add_plugin(TnuaRapier3dPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(StarfieldPlugin)
        .add_plugin(MapPlugin)
        .add_system(Dolly::<MainCamera>::update_active)
        .run();
}

/// set up a simple 3D scene
fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_translation(map_to_world(START_TILE) + Vec3::Y * 0.5),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity::default(),
            Collider::capsule_y(0.5, 0.5),
            TnuaPlatformerBundle::new_with_config(TnuaPlatformerConfig {
                full_speed: 6.0,
                full_jump_height: 2.0,
                up: Vec3::Y,
                forward: -Vec3::Z,
                float_height: 1.0,
                cling_distance: 1.0,
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
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, -0.5, 0.0),
                ..default()
            });

            // cursor
            parent.spawn((
                Cursor,
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

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -1.0, -1.0, -1.0)),
        ..default()
    });
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

fn apply_controls(keyboard: Res<Input<KeyCode>>, mut query: Query<&mut TnuaPlatformerControls>) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::Up) || keyboard.pressed(KeyCode::W) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D) {
        direction += Vec3::X;
    }

    let jump = keyboard.pressed(KeyCode::Space);

    let turn_in_place = [KeyCode::LAlt, KeyCode::RAlt]
        .into_iter()
        .any(|key_code| keyboard.pressed(key_code));

    for mut controls in query.iter_mut() {
        *controls = TnuaPlatformerControls {
            desired_velocity: if turn_in_place { Vec3::ZERO } else { direction },
            desired_forward: direction.normalize(),
            jump: jump.then(|| 1.0),
        };
    }
}

fn update_camera(q0: Query<(&Transform, With<Player>)>, mut q1: Query<&mut Rig>) {
    let player = q0.single().0.to_owned();
    let mut rig = q1.single_mut();

    rig.driver_mut::<Position>().position = player.translation;
    // rig.driver_mut::<Rotation>().rotation = player.rotation;
    rig.driver_mut::<LookAt>().target = player.translation + Vec3::Y;
}

fn cursor(
    mut collision_events: EventReader<CollisionEvent>,
    cursor_query: Query<Entity, With<Cursor>>,
    mut floor_query: Query<&mut Handle<StandardMaterial>, With<Floor>>,
    floor_materials: Res<FloorMaterials>,
) {
    for evt in collision_events.iter() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let is_cursor = cursor_query.iter_many([e1, e2]).count() > 0;
                if !is_cursor {
                    continue;
                }

                let mut iter = floor_query.iter_many_mut([e1, e2]);
                while let Some(mut floor_material) = iter.fetch_next() {
                    *floor_material = floor_materials.highlighted.clone();
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let is_cursor = cursor_query.iter_many([e1, e2]).count() > 0;
                if !is_cursor {
                    continue;
                }

                let mut iter = floor_query.iter_many_mut([e1, e2]);
                while let Some(mut floor_material) = iter.fetch_next() {
                    *floor_material = floor_materials.normal.clone();
                }
            }
        }
    }
}
