use bevy::audio::Volume;
use bevy::math::Vec3Swizzles;
use bevy::{prelude::*, utils::HashSet};
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineVolume};
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use bevy_two_entities::tuple::TupleQueryExt;

use crate::enemy::{HitPoints, PathIndex};
use crate::loading::Sounds;
use crate::map::{PlacedTower, TilePos, PATH};
use crate::settings::SfxSetting;
use crate::DespawnOnReset;
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
pub struct SpawnTowerEvent(pub Entity);

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
pub struct LaserMaterial(pub Handle<StandardMaterial>);
impl FromWorld for LaserMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(StandardMaterial {
            base_color: bevy::color::palettes::basic::YELLOW.into(),
            emissive: bevy::color::palettes::basic::WHITE.into(),
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
    mut tower_query: Query<(Entity, &Target, &Transform), (Without<Enemy>, Without<TowerHead>)>,
    mut tower_head_query: Query<&mut Transform, With<TowerHead>>,
    target_query: Query<&Transform, (With<Enemy>, Without<TowerHead>)>,
    time: Res<Time>,
    children_query: Query<&Children>,
) {
    for (entity, target, transform) in tower_query.iter_mut() {
        let Some(target_entity) = target.0 else {
            continue;
        };

        let Ok(target_transform) = target_query.get(target_entity) else {
            continue;
        };

        let diff_xz = (transform.translation - target_transform.translation).xz();

        for descendant in children_query.iter_descendants(entity) {
            if let Ok(mut head) = tower_head_query.get_mut(descendant) {
                head.rotation = head.rotation.slerp(
                    Quat::from_rotation_y(diff_xz.angle_to(-Vec2::Y)),
                    time.delta_secs() * 10.,
                );

                break;
            }
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
                let Some((range_sensor_parent, enemy_entity)) = queries.get_both(*e1, *e2) else {
                    continue;
                };

                if let Ok(mut in_range) = tower_query.get_mut(range_sensor_parent.get()) {
                    in_range.0.insert(enemy_entity);
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let queries = (&range_sensor_query, &enemy_query);
                let Some((range_sensor_parent, enemy_entity)) = queries.get_both(*e1, *e2) else {
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

fn spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnTowerEvent>,
    tile_pos_query: Query<&TilePos>,
    models: Res<Models>,
) {
    for event in events.read() {
        let Ok(tile_pos) = tile_pos_query.get(event.0) else {
            continue;
        };

        let entity = commands
            .spawn((
                Tower,
                Name::new("Tower"),
                HookedSceneBundle {
                    scene: SceneRoot(models.tower_base.clone()),
                    hook: SceneHook::new(|entity, cmds| {
                        match entity.get::<Name>().map(|t| t.as_str()) {
                            Some("HeadMesh") => {
                                cmds.insert(TowerHead);
                                cmds
                            }
                            _ => cmds,
                        };
                    }),
                },
                Transform::from_translation(map_to_world(tile_pos.0) + Vec3::Y * 0.75),
                Target(None),
                InRange::default(),
                Cooldown(Timer::from_seconds(2.5, TimerMode::Repeating)),
                TilePos(tile_pos.0),
                Ammo::new(20),
                RigidBody::Fixed,
                Collider::cuboid(1.0, 3.0, 1.0),
                ActiveEvents::COLLISION_EVENTS,
                OutlineVolume {
                    width: 3.0,
                    colour: Color::hsla(160., 0.9, 0.5, 1.0),
                    visible: true,
                },
                AsyncSceneInheritOutline::default(),
                DespawnOnReset,
            ))
            .with_children(|parent| {
                parent.spawn((
                    RangeSensor,
                    Transform::default(),
                    Visibility::default(),
                    Collider::ball(4.),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                ));
            })
            .id();

        commands.entity(event.0).insert(PlacedTower(entity));
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

    commands.spawn((
        AudioPlayer(game_audio.build.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::new(**audio_setting as f32 / 100.)),
    ));
}

fn shooting(
    mut commands: Commands,
    mut tower_query: Query<(Entity, &mut Cooldown, &mut Ammo, &Target), With<Tower>>,
    tower_head_query: Query<&GlobalTransform, With<TowerHead>>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<LaserMaterial>,
    time: Res<Time>,
    children_query: Query<&Children>,
) {
    let offset = Vec3::new(0., -0.2, 0.8);

    for (entity, mut cooldown, mut ammo, target) in tower_query.iter_mut() {
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

        let Some(head) = children_query
            .iter_descendants(entity)
            .find_map(|descendant| tower_head_query.get(descendant).ok())
        else {
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
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
            MeshMaterial3d(material.0.clone()),
            laser_transform,
            Target(target.0),
            DespawnOnReset,
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
            let step = time.delta_secs() * 8.;

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
            commands.spawn((
                AudioPlayer(game_audio.powerdown.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::new(**audio_setting as f32 / 100.)),
            ));
        }
    }
}
