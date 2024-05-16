use bevy::{audio::Volume, math::Vec3Swizzles, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    loading::{Models, Sounds},
    map::{map_to_world, PATH},
    settings::SfxSetting,
    GameState, Lives,
};

pub struct EnemyPlugin;

#[derive(Component)]
pub struct PathIndex(pub usize);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
}
impl HitPoints {
    /// Creates a new `HitPoints`, starting with full health.
    fn new(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub hp: u32,
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>()
            .add_systems(Update, spawn.run_if(in_state(GameState::Playing)))
            .add_systems(Update, movement.run_if(in_state(GameState::Playing)))
            .add_systems(Update, death.run_if(in_state(GameState::Playing)));
    }
}

fn spawn(mut commands: Commands, models: Res<Models>, mut events: EventReader<SpawnEnemyEvent>) {
    for event in events.read() {
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
            HitPoints::new(event.hp),
        ));
    }
}

fn movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut PathIndex), With<Enemy>>,
    time: Res<Time>,
    mut lives: ResMut<Lives>,
    game_audio: Res<Sounds>,
    audio_setting: Res<SfxSetting>,
) {
    for (entity, mut transform, mut path_index) in query.iter_mut() {
        if let Some(next_waypoint) = PATH.get(path_index.0 + 1) {
            let world = map_to_world(*next_waypoint);

            let diff = world - transform.translation;
            let dist = diff.length();

            let step = 1. * time.delta_seconds();

            let diff_xz = diff.xz();
            transform.rotation = Quat::from_rotation_y(diff_xz.angle_between(Vec2::Y));
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
            commands.spawn(AudioBundle {
                source: game_audio.damage.clone(),
                settings: PlaybackSettings::DESPAWN
                    .with_volume(Volume::new(**audio_setting as f32 / 100.)),
            });

            lives.0 = lives.0.saturating_sub(1);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn death(mut commands: Commands, query: Query<(Entity, &HitPoints), With<Enemy>>) {
    for (entity, hp) in query.iter() {
        if hp.current == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
