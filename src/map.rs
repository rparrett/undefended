use std::{f32::consts::FRAC_PI_2, fmt::Display, time::Duration};

use bevy::{prelude::*, utils::HashSet};
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineVolume};
use bevy_rapier3d::prelude::*;
use bevy_tnua::TnuaPipelineStages;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{loading::Models, DespawnOnReset, GameState};

pub struct MapPlugin;

const MAP: [[i32; 15]; 15] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 0],
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 1, 1, 4, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];
const MAP_ROWS: usize = MAP.len();
const MAP_COLS: usize = MAP[0].len();
const TILE_SIZE: Vec3 = Vec3::new(2., 0.5, 2.);
const LAVA_DEPTH: f32 = -20.;
pub const PATH: [UVec2; 8] = [
    UVec2::new(0, 4),
    UVec2::new(11, 4),
    UVec2::new(11, 10),
    UVec2::new(6, 10),
    UVec2::new(6, 9),
    UVec2::new(3, 9),
    UVec2::new(3, 7),
    UVec2::new(0, 7),
];
pub const START_TILE: UVec2 = UVec2::new(12, 11);

#[derive(Component)]
pub struct Floor;

#[derive(Component)]
pub struct TilePos(pub UVec2);

#[derive(Component)]
pub struct Lava;

#[derive(Component)]
pub struct ItemSpawner {
    pub item: Item,
    pub timer: Timer,
    pub spawned: usize,
}
impl ItemSpawner {
    fn new(item: Item, secs: f32) -> Self {
        let mut timer = Timer::new(Duration::from_secs_f32(secs), TimerMode::Once);

        // make the first tick finish the timer
        timer.set_elapsed(Duration::from_secs_f32(secs - f32::EPSILON));

        Self {
            item,
            timer,
            spawned: 0,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    TowerKit,
    LaserAmmo,
}
impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Item::TowerKit => "Tower Kit",
                Item::LaserAmmo => "Ammo",
            }
        )
    }
}

#[derive(Component)]
pub struct PlacedTower(pub Entity);

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

#[derive(Resource)]
pub struct PathMaterial(pub Handle<StandardMaterial>);
impl FromWorld for PathMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(StandardMaterial {
            base_color: Srgba::new(1.0, 0.0, 0.0, 0.3).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }))
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PathMaterial>()
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            // This must run before `TnuaPipelineStages::Sensors` or the player's movement
            // will not match up with the moving platform.
            .add_systems(
                Update,
                moving_floor
                    .run_if(in_state(GameState::Playing))
                    .before(TnuaPipelineStages::Sensors),
            )
            .add_systems(Update, item_spawner.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                item_spawner_reset.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                item_idle_movement.run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn map_to_world(pos: UVec2) -> Vec3 {
    Vec3::new(
        (MAP_COLS as i32 / -2 + pos.x as i32) as f32 * TILE_SIZE.x,
        0.,
        (MAP_ROWS as i32 / -2 + pos.y as i32) as f32 * TILE_SIZE.z,
    )
}

fn spawn_map(
    mut commands: Commands,
    models: Res<Models>,
    mut meshes: ResMut<Assets<Mesh>>,
    path_material: Res<PathMaterial>,
) {
    let mut rng = thread_rng();

    let handles = [&models.tile1, &models.tile2, &models.tile3, &models.tile4];
    let item_spawner_handle = &models.item_spawner;

    for (row, row_val) in MAP.iter().enumerate() {
        for (col, col_val) in row_val.iter().enumerate() {
            if *col_val == 0 {
                continue;
            }

            if *col_val == 2 {
                continue;
            }

            let pos = UVec2::new(col as u32, row as u32);

            let handle = if *col_val == 3 || *col_val == 4 {
                item_spawner_handle
            } else {
                *handles.choose(&mut rng).unwrap()
            };

            let mut cmds = commands.spawn((
                Floor,
                SceneRoot(handle.clone()),
                Transform::from_translation(map_to_world(pos) + Vec3::Y * -0.5).with_rotation(
                    Quat::from_rotation_y(rng.gen_range(0..3) as f32 * FRAC_PI_2),
                ),
                TilePos(pos),
                Collider::cuboid(TILE_SIZE.x / 2., TILE_SIZE.y / 2., TILE_SIZE.x / 2.),
                ActiveEvents::COLLISION_EVENTS,
                OutlineVolume {
                    width: 3.0,
                    colour: Color::hsla(160., 0.9, 0.5, 1.0),
                    visible: true,
                },
                AsyncSceneInheritOutline::default(),
                DespawnOnReset,
            ));

            if *col_val == 3 {
                cmds.insert(ItemSpawner::new(Item::TowerKit, 30.0));
                cmds.insert(Name::new("TowerKitSpawner"));
            } else if *col_val == 4 {
                cmds.insert(ItemSpawner::new(Item::LaserAmmo, 2.0));
                cmds.insert(Name::new("LaserAmmoSpawner"));
            } else {
                cmds.insert(Name::new("Floor"));
            }

            // XXX
            if col == 5 && row == 12 {
                cmds.insert((
                    MovingFloor {
                        waypoints: vec![UVec2::new(5, 12), UVec2::new(10, 12)],
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

    for window in PATH.windows(2) {
        let (start, end) = (window[0], window[1]);

        let mut xs = [start.x, end.x];
        xs.sort();
        let [start_x, end_x] = xs;

        let mut ys = [start.y, end.y];
        ys.sort();
        let [start_y, end_y] = ys;

        let mut path_tiles: HashSet<UVec2> = HashSet::default();

        for col in start_x..=end_x {
            for row in start_y..=end_y {
                path_tiles.insert(UVec2::new(col, row));
            }
        }

        for path_tile in path_tiles {
            commands.spawn((
                Name::new("PathDot"),
                Mesh3d(meshes.add(Cuboid::new(0.25, 0.25, 0.25))),
                MeshMaterial3d(path_material.0.clone()),
                Transform::from_translation(map_to_world(path_tile)),
                DespawnOnReset,
            ));
        }
    }

    commands.spawn((
        Lava,
        Transform::from_translation(Vec3::new(0., LAVA_DEPTH, 0.)),
        Collider::halfspace(Vec3::Y).unwrap(),
        Sensor,
        DespawnOnReset,
    ));
}

fn moving_floor(
    mut query: Query<(&Transform, &mut Velocity, &mut MovingFloor), With<MovingFloor>>,
    time: Res<Time>,
) {
    for (transform, mut velocity, mut floor) in query.iter_mut() {
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
                    let diff = world - transform.translation;
                    let dist = diff.length();

                    // This is slightly janky.
                    //
                    // It would be nice for the platform to be a RigidBody::KinematicPositionBased,
                    // but this is incompatible with `bevy_tnua`.
                    //
                    // See https://github.com/idanarye/bevy-tnua/issues/28.

                    let step = floor.speed * time.delta_secs();
                    let adjusted_speed = floor.speed * (dist / step).min(1.);

                    if dist < 0.01 {
                        velocity.linvel = Vec3::ZERO;
                        floor.advance();
                    } else {
                        velocity.linvel = diff.normalize_or_zero() * adjusted_speed;
                    }
                } else {
                    floor.state = MovingFloorState::Dwell;
                    floor.direction.toggle();
                }
            }
        }
    }
}

fn item_spawner(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ItemSpawner)>,
    time: Res<Time>,
    models: Res<Models>,
) {
    for (entity, mut item_spawner) in query.iter_mut() {
        item_spawner.timer.tick(time.delta());
        if !item_spawner.timer.just_finished() {
            continue;
        }

        item_spawner.spawned += 1;

        let item = commands
            .spawn((
                item_spawner.item,
                Name::new("Item"),
                SceneRoot(match item_spawner.item {
                    Item::LaserAmmo => models.laser_ammo.clone(),
                    Item::TowerKit => models.tower_kit.clone(),
                }),
                Transform::from_xyz(0., 1.0, 0.),
                Collider::ball(0.6),
                Sensor,
                OutlineVolume {
                    width: 3.0,
                    colour: Color::hsla(160., 0.9, 0.5, 1.0),
                    visible: true,
                },
                AsyncSceneInheritOutline::default(),
            ))
            .id();

        commands.entity(entity).add_child(item);
    }
}

fn item_spawner_reset(
    mut query: Query<(&mut ItemSpawner, &Children), Changed<Children>>,
    item_query: Query<(), With<Item>>,
) {
    for (mut item_spawner, children) in query.iter_mut() {
        if item_query.iter_many(children).count() == 0 {
            item_spawner.timer.reset();
        }
    }
}

fn item_idle_movement(
    mut query: Query<(&Parent, &mut Transform), With<Item>>,
    spawner_query: Query<&ItemSpawner>,
    time: Res<Time>,
) {
    for (parent, mut transform) in query.iter_mut() {
        // Only animate items while they are on the spawner
        if spawner_query.get(**parent).is_err() {
            continue;
        }

        transform.translation.y = 1.0 + (time.elapsed_secs_wrapped() * 2.).sin() * 0.05;
        transform.rotate_y(time.delta_secs() * 0.3);
    }
}
