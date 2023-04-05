use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::GameState;

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
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];
const MAP_ROWS: usize = MAP.len();
const MAP_COLS: usize = MAP[0].len();
const TILE_SIZE: Vec3 = Vec3::new(2., 0.5, 2.);

#[derive(Component)]
pub struct Floor;

#[derive(Resource)]
pub struct FloorMaterials {
    pub normal: Handle<StandardMaterial>,
    pub highlighted: Handle<StandardMaterial>,
}
impl FromWorld for FloorMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        Self {
            normal: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.5),
                ..default()
            }),
            highlighted: materials.add(StandardMaterial {
                base_color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            }),
        }
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloorMaterials>()
            .add_system(spawn_map.in_schedule(OnEnter(GameState::Playing)));
    }
}

pub fn map_to_world(pos: UVec2) -> Vec3 {
    return Vec3::new(
        (MAP_COLS as i32 / -2 + pos.x as i32) as f32 * TILE_SIZE.x,
        0.,
        (MAP_ROWS as i32 / -2 + pos.y as i32) as f32 * TILE_SIZE.z,
    );
}

fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    floor_materials: Res<FloorMaterials>,
) {
    let tile_mesh = meshes.add(shape::Box::new(TILE_SIZE.x, TILE_SIZE.y, TILE_SIZE.x).into());

    for (row, row_val) in MAP.iter().enumerate() {
        for (col, col_val) in row_val.iter().enumerate() {
            if *col_val == 1 {
                commands.spawn((
                    Floor,
                    PbrBundle {
                        transform: Transform::from_translation(
                            map_to_world(UVec2::new(col as u32, row as u32)) + Vec3::Y * -0.5,
                        ),
                        mesh: tile_mesh.clone(),
                        material: floor_materials.normal.clone(),
                        ..default()
                    },
                    Collider::cuboid(TILE_SIZE.x / 2., TILE_SIZE.y / 2., TILE_SIZE.x / 2.),
                    ActiveEvents::COLLISION_EVENTS,
                ));
            }
        }
    }
}
