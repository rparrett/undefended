use bevy::audio::Volume;
use bevy::math::Vec3Swizzles;
use bevy::{prelude::*, utils::HashSet};
use bevy_rapier3d::prelude::*;

use crate::enemy::{HitPoints, PathIndex};
use crate::loading::Sounds;
use crate::map::{TilePos, PATH};
use crate::settings::SfxSetting;
use crate::{enemy::Enemy, loading::Models, map::map_to_world, GameState};

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct TowerHead;

#[derive(Component, Debug)]
pub struct Target(pub Option<Entity>);

#[derive(Component, Debug, Default)]
pub struct InRange(pub HashSet<Entity>);

#[derive(Event)]
pub struct SpawnTowerEvent(pub UVec2);

#[derive(Component)]
pub struct RangeSensor;

#[derive(Component)]
struct Laser;

#[derive(Component)]
struct Cooldown(Timer);

#[derive(Component)]
pub struct Ammo {
    pub current: u32,
    pub max: u32,
}
impl Ammo {
    /// Creates a new `Ammo`, starting with full ammo.
    fn new(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Resource)]
struct LaserMaterial(Handle<StandardMaterial>);
impl FromWorld for LaserMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            emissive: Color::WHITE,
            unlit: true,
            ..default()
        }))
    }
}

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTowerEvent>()
            .init_resource::<LaserMaterial>()
            .add_systems(Update, spawn.run_if(in_state(GameState::Playing)))
            .add_systems(Update, ranging.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                targeting
                    .run_if(in_state(GameState::Playing))
                    .after(ranging),
            )
            .add_systems(
                Update,
                shooting
                    .run_if(in_state(GameState::Playing))
                    .after(targeting),
            )
            .add_systems(Update, movement.run_if(in_state(GameState::Playing)))
            .add_systems(Update, laser_movement.run_if(in_state(GameState::Playing)))
            .add_systems(Update, build_sound.run_if(in_state(GameState::Playing)))
            .add_systems(Update, laser_sound.run_if(in_state(GameState::Playing)));
    }
}

fn movement(
    mut tower_query: Query<(&Target, &Transform, &Children), (Without<Enemy>, Without<TowerHead>)>,
    mut tower_head_query: Query<&mut Transform, With<TowerHead>>,
    target_query: Query<&Transform, (With<Enemy>, Without<TowerHead>)>,
    time: Res<Time>,
) {
    for (target, transform, children) in tower_query.iter_mut() {
        let Some(target_entity) = target.0 else {
            continue;
        };

        let Ok(target_transform) = target_query.get(target_entity) else {
            continue;
        };

        let diff_xz = (transform.translation - target_transform.translation).xz();

        let mut iter = tower_head_query.iter_many_mut(children);
        while let Some(mut tower_head) = iter.fetch_next() {
            tower_head.rotation = tower_head.rotation.slerp(
                Quat::from_rotation_y(diff_xz.angle_between(-Vec2::Y)),
                time.delta_seconds() * 10.,
            );
        }
    }
}

fn ranging(
    mut collision_events: EventReader<CollisionEvent>,
    range_sensor_query: Query<&Parent, With<RangeSensor>>,
    mut tower_query: Query<&mut InRange, With<Tower>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for evt in collision_events.read() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let queries = (&range_sensor_query, &enemy_query);
                let (Some(range_sensor_parent), Some(enemy_entity)) = queries.get_both() else {
                    continue;
                };

                if let Ok(mut in_range) = tower_query.get_mut(range_sensor_parent.get()) {
                    in_range.0.insert(enemy_entity);
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let queries = (&range_sensor_query, &enemy_query);
                let (Some(range_sensor_parent), Some(enemy_entity)) = queries.get_both() else {
                    continue;
                };

                if let Ok(mut in_range) = tower_query.get_mut(range_sensor_parent.get()) {
                    in_range.0.remove(&enemy_entity);
                }
            }
        }
    }
}

fn targeting(
    mut tower_query: Query<(&mut Target, &InRange), With<Tower>>,
    enemy_query: Query<(Entity, &Transform, &PathIndex), With<Enemy>>,
) {
    for (mut target, in_range) in tower_query.iter_mut() {
        // Don't pick a new target if we already have one and
        // it is a valid enemy reference and the enemy is still
        // in range.
        if target
            .0
            .filter(|t| in_range.0.contains(t))
            .filter(|t| enemy_query.get(*t).is_ok())
            .is_some()
        {
            continue;
        }

        // TODO we should sort by path index before distance to destination
        let mut enemies: Vec<_> = enemy_query
            .iter_many(&in_range.0)
            .map(|(entity, transform, path_index)| {
                let dist =
                    (transform.translation - map_to_world(PATH[path_index.0])).length_squared();
                (entity, dist)
            })
            .collect();

        enemies.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if let Some((entity, _)) = enemies.first() {
            target.0 = Some(*entity);
        } else {
            target.0 = None;
        }
    }
}

fn spawn(mut commands: Commands, mut events: EventReader<SpawnTowerEvent>, models: Res<Models>) {
    for event in events.read() {
        commands
            .spawn((
                Tower,
                Name::new("Tower"),
                SceneBundle {
                    scene: models.tower_base.clone(),
                    transform: Transform::from_translation(map_to_world(event.0) + Vec3::Y * 0.75),
                    ..default()
                },
                Target(None),
                InRange::default(),
                Cooldown(Timer::from_seconds(2.5, TimerMode::Repeating)),
                TilePos(event.0),
                Ammo::new(20),
                RigidBody::Fixed,
                Collider::cuboid(1.0, 3.0, 1.0),
                ActiveEvents::COLLISION_EVENTS,
            ))
            .with_children(|parent| {
                parent.spawn((
                    TowerHead,
                    Name::new("TowerHead"),
                    SceneBundle {
                        scene: models.tower_head.clone(),
                        transform: Transform::from_translation(Vec3::Y * 1.5),
                        ..default()
                    },
                ));

                parent.spawn((
                    RangeSensor,
                    SpatialBundle::default(),
                    Collider::ball(4.),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                ));
            });
    }
}

fn build_sound(
    mut commands: Commands,
    mut events: EventReader<SpawnTowerEvent>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
) {
    if events.read().count() == 0 {
        return;
    }

    commands.spawn(AudioBundle {
        source: game_audio.build.clone(),
        settings: PlaybackSettings::ONCE
            .with_volume(Volume::new_absolute(**audio_setting as f32 / 100.)),
    });
}

fn shooting(
    mut commands: Commands,
    mut tower_query: Query<(&mut Cooldown, &mut Ammo, &Target, &Children), With<Tower>>,
    tower_head_query: Query<&GlobalTransform, With<TowerHead>>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<LaserMaterial>,
    time: Res<Time>,
) {
    let offset = Vec3::new(0., -0.2, 0.8);

    for (mut cooldown, mut ammo, target, children) in tower_query.iter_mut() {
        cooldown.0.tick(time.delta());
        if !cooldown.0.just_finished() {
            continue;
        }

        if target.0.is_none() {
            continue;
        }

        if ammo.current == 0 {
            continue;
        }

        let Some(head) = tower_head_query.iter_many(children).next() else {
            warn!("headless tower?");
            continue;
        };

        let (scale, rotation, translation) = head.to_scale_rotation_translation();
        let laser_transform = Transform {
            scale,
            rotation,
            translation: translation + rotation.mul_vec3(offset),
        };

        ammo.current = ammo.current.saturating_sub(1);

        commands.spawn((
            Laser,
            Name::new("Laser"),
            PbrBundle {
                transform: laser_transform,
                mesh: meshes.add(shape::Box::new(0.1, 0.1, 0.1).into()),
                material: material.0.clone(),
                ..default()
            },
            Target(target.0),
        ));
    }
}

fn laser_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Target), With<Laser>>,
    mut enemy_query: Query<(&mut HitPoints, &Transform), Without<Laser>>,
    time: Res<Time>,
) {
    for (laser_entity, mut transform, target) in query.iter_mut() {
        let Some(target_entity) = target.0 else {
            continue;
        };

        if let Ok((mut hp, enemy)) = enemy_query.get_mut(target_entity) {
            let diff = enemy.translation - transform.translation;
            let dist = diff.length();
            let step = time.delta_seconds() * 8.;

            if dist > step {
                transform.translation += step * diff.normalize();
            } else {
                hp.current = hp.current.saturating_sub(1);
                commands.entity(laser_entity).despawn_recursive();
            }
        } else {
            commands.entity(laser_entity).despawn_recursive();
        }
    }
}

fn laser_sound(
    mut commands: Commands,
    query: Query<&Ammo, Changed<Ammo>>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
) {
    for ammo in query.iter() {
        if ammo.current == 0 {
            commands.spawn(AudioBundle {
                source: game_audio.powerdown.clone(),
                settings: PlaybackSettings::ONCE
                    .with_volume(Volume::new_absolute(**audio_setting as f32 / 100.)),
            });
        }
    }
}
