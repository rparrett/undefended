use bevy::math::Vec3Swizzles;
use bevy::{prelude::*, utils::HashSet};
use bevy_rapier3d::prelude::*;

use crate::enemy::PathIndex;
use crate::map::PATH;
use crate::{enemy::Enemy, loading::Models, map::map_to_world, GameState};

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct TowerHead;

#[derive(Component)]
pub struct Target(pub Option<Entity>);

#[derive(Component, Default)]
pub struct InRange(pub HashSet<Entity>);

pub struct SpawnTowerEvent(pub UVec2);

#[derive(Component)]
pub struct RangeSensor;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTowerEvent>()
            .add_system(spawn.in_set(OnUpdate(GameState::Playing)))
            .add_system(ranging.in_set(OnUpdate(GameState::Playing)))
            .add_system(
                targeting
                    .in_set(OnUpdate(GameState::Playing))
                    .after(ranging),
            )
            .add_system(movement.in_set(OnUpdate(GameState::Playing)));
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
            continue
        };

        let Ok(target_transform) = target_query.get(target_entity) else {
            continue
        };

        // TODO use y for gun barrels
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
    for evt in collision_events.iter() {
        match evt {
            CollisionEvent::Started(e1, e2, _) => {
                let is_range_sensor = range_sensor_query.iter_many([e1, e2]).count() > 0;
                let is_enemy = enemy_query.iter_many([e1, e2]).count() > 0;

                if is_range_sensor && is_enemy {
                    for parent in range_sensor_query.iter_many([e1, e2]) {
                        if let Ok(mut in_range) = tower_query.get_mut(parent.get()) {
                            for entity in enemy_query.iter_many([e1, e2]) {
                                in_range.0.insert(entity);
                            }
                        }
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let is_range_sensor = range_sensor_query.iter_many([e1, e2]).count() > 0;
                let is_enemy = enemy_query.iter_many([e1, e2]).count() > 0;

                if is_range_sensor && is_enemy {
                    for parent in range_sensor_query.iter_many([e1, e2]) {
                        if let Ok(mut in_range) = tower_query.get_mut(parent.get()) {
                            for entity in enemy_query.iter_many([e1, e2]) {
                                in_range.0.remove(&entity);
                            }
                        }
                    }
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
            .map(|t| enemy_query.get(t))
            .is_some()
        {
            continue;
        }

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
    // TODO remove tile from LastTile

    for event in events.iter() {
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
