use bevy::{prelude::*, utils::Duration};
use bevy_rapier3d::prelude::*;

use crate::{
    loading::Models,
    map::{map_to_world, PATH},
    GameState,
};

pub struct EnemyPlugin;

#[derive(Resource)]
struct EnemyTimer(Timer);
impl Default for EnemyTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(4.),
            TimerMode::Repeating,
        ))
    }
}

#[derive(Component)]
pub struct PathIndex(pub usize);

#[derive(Component)]
pub struct Enemy;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyTimer>()
            .add_system(spawn.in_set(OnUpdate(GameState::Playing)))
            .add_system(movement.in_set(OnUpdate(GameState::Playing)));
    }
}

fn spawn(
    mut commands: Commands,
    mut timer: ResMut<EnemyTimer>,
    time: Res<Time>,
    models: Res<Models>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    commands.spawn((
        Enemy,
        Name::new("Enemy"),
        SceneBundle {
            scene: models.enemy1.clone(),
            transform: Transform::from_translation(map_to_world(PATH[0])),
            ..default()
        },
        Collider::ball(0.5),
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::STATIC_STATIC,
        Sensor,
        PathIndex(0),
    ));
}

fn movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut PathIndex), With<Enemy>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut path_index) in query.iter_mut() {
        if let Some(next_waypoint) = PATH.get(path_index.0 + 1) {
            let world = map_to_world(*next_waypoint);

            let diff = world - transform.translation;
            let dist = diff.length();

            let step = 1. * time.delta_seconds();

            transform.rotation = Quat::from_rotation_y(diff.angle_between(Vec3::Z));
            transform.rotate_local_z(time.elapsed_seconds());

            if step < dist {
                transform.translation.x += step / dist * (world.x - transform.translation.x);
                transform.translation.z += step / dist * (world.z - transform.translation.z);
            } else {
                transform.translation.x = world.x;
                transform.translation.z = world.z;
                path_index.0 += 1;
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}
