use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};

use crate::{GameState, Player};

pub struct StarfieldPlugin;
#[derive(Component)]
struct Starfield;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<StarfieldMaterial>::default())
            .add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(move_starfield.in_set(OnUpdate(GameState::Playing)));
    }
}

fn setup(
    mut commands: Commands,
    mut mat2d: ResMut<Assets<StarfieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();

    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: -1,
            ..default()
        },
        ..default()
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(window.width(), window.height())).into())
                .into(),
            material: mat2d.add(StarfieldMaterial::default()),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        Starfield,
    ));

    info!("spawning starfield");
}

fn move_starfield(
    query: Query<&Transform, With<Player>>,
    mut materials: ResMut<Assets<StarfieldMaterial>>,
) {
    for player in query.iter() {
        for mat in materials.iter_mut() {
            mat.1.pos = Vec2::new(player.translation.x, player.translation.z);
        }
    }
}

impl Material2d for StarfieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Default, Clone)]
#[uuid = "1e0463f0-c315-4d84-bf54-f7a1abf93ff5"]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub pos: Vec2,
    #[uniform(0)]
    pub _wasm_padding: Vec2,
}
