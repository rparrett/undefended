use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{loading::Models, GameState};

pub struct MapPlugin;

const MAP: [[i32; 13]; 13] = [
    [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0, 2, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];
const MAP_ROWS: usize = MAP.len();
const MAP_COLS: usize = MAP[0].len();
const TILE_SIZE: Vec3 = Vec3::new(2., 0.5, 2.);
const LAVA_DEPTH: f32 = -20.;

#[derive(Component)]
pub struct Floor;

#[derive(Component)]
pub struct TilePos(pub UVec2);

#[derive(Component)]
pub struct Lava;

#[derive(Component)]
pub struct MovingFloor {
    waypoints: Vec<UVec2>,
    index: usize,
    speed: f32,
    dwell_timer: Timer,
    state: MovingFloorState,
    direction: MovingFloorDirection,
}
impl MovingFloor {
    fn next_waypoint(&self) -> Option<UVec2> {
        let index = self.next_waypoint_index()?;

        self.waypoints.get(index).copied()
    }

    fn next_waypoint_index(&self) -> Option<usize> {
        match self.direction {
            MovingFloorDirection::Forward => {
                self.waypoints.get(self.index + 1).and(Some(self.index + 1))
            }
            MovingFloorDirection::Backward => self.index.checked_sub(1),
        }
    }

    fn advance(&mut self) {
        if let Some(index) = self.next_waypoint_index() {
            self.index = index;
        }
    }
}
enum MovingFloorState {
    Dwell,
    Move,
}
enum MovingFloorDirection {
    Forward,
    Backward,
}
impl MovingFloorDirection {
    fn toggle(&mut self) {
        *self = match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_map.in_schedule(OnEnter(GameState::Playing)))
            .add_system(moving_floor.in_set(OnUpdate(GameState::Playing)));
    }
}

pub fn map_to_world(pos: UVec2) -> Vec3 {
    Vec3::new(
        (MAP_COLS as i32 / -2 + pos.x as i32) as f32 * TILE_SIZE.x,
        0.,
        (MAP_ROWS as i32 / -2 + pos.y as i32) as f32 * TILE_SIZE.z,
    )
}

fn spawn_map(mut commands: Commands, models: Res<Models>) {
    let mut rng = thread_rng();

    let handles = [&models.tile1, &models.tile2, &models.tile3, &models.tile4];

    for (row, row_val) in MAP.iter().enumerate() {
        for (col, col_val) in row_val.iter().enumerate() {
            if *col_val == 1 {
                let pos = UVec2::new(col as u32, row as u32);

                let handle = *handles.choose(&mut rng).unwrap();

                let mut cmds = commands.spawn((
                    Floor,
                    SceneBundle {
                        scene: handle.clone(),
                        transform: Transform::from_translation(map_to_world(pos) + Vec3::Y * -0.5)
                            .with_rotation(Quat::from_rotation_y(
                                rng.gen_range(0..3) as f32 * FRAC_PI_2,
                            )),
                        ..default()
                    },
                    TilePos(pos),
                    Collider::cuboid(TILE_SIZE.x / 2., TILE_SIZE.y / 2., TILE_SIZE.x / 2.),
                    ActiveEvents::COLLISION_EVENTS,
                ));

                if col == 3 && row == 10 {
                    cmds.insert((
                        MovingFloor {
                            waypoints: vec![UVec2::new(3, 10), UVec2::new(8, 10)],
                            index: 0,
                            speed: 5.,
                            state: MovingFloorState::Dwell,
                            direction: MovingFloorDirection::Forward,
                            dwell_timer: Timer::new(Duration::from_secs_f32(1.), TimerMode::Once),
                        },
                        RigidBody::KinematicVelocityBased,
                        Velocity::default(),
                    ));
                }
            }
        }
    }

    commands.spawn((
        Lava,
        SpatialBundle::from_transform(Transform::from_translation(Vec3::new(0., LAVA_DEPTH, 0.))),
        Collider::halfspace(Vec3::Y).unwrap(),
        Sensor,
    ));
}

fn moving_floor(
    mut query: Query<(&mut Velocity, &Transform, &mut MovingFloor), With<MovingFloor>>,
    time: Res<Time>,
) {
    for (mut velocity, transform, mut floor) in query.iter_mut() {
        match floor.state {
            MovingFloorState::Dwell => {
                floor.dwell_timer.tick(time.delta());
                if floor.dwell_timer.just_finished() {
                    floor.state = MovingFloorState::Move;
                    floor.dwell_timer.reset();
                }
            }
            MovingFloorState::Move => {
                if let Some(next_waypoint) = floor.next_waypoint() {
                    let world = map_to_world(next_waypoint) + Vec3::Y * -0.5;
                    let delta = world - transform.translation;
                    let dist = delta.length();

                    // TODO acceleration
                    if dist > 0.01 {
                        velocity.linvel = delta.normalize_or_zero() * floor.speed;
                    } else {
                        velocity.linvel = Vec3::ZERO;
                        floor.advance();
                    }
                } else {
                    floor.state = MovingFloorState::Dwell;
                    velocity.linvel = Vec3::ZERO;
                    floor.direction.toggle();
                }
            }
        }
    }
}
