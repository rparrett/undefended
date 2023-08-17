use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};

use crate::{GameState, Persist, Player};

pub struct StarfieldPlugin;
#[derive(Component)]
struct Starfield;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<StarfieldMaterial>::default())
            .add_systems(OnEnter(GameState::Pipelines), setup)
            .add_systems(Update, move_starfield.run_if(in_state(GameState::Playing)));
    }
}

fn setup(
    mut commands: Commands,
    mut mat2d: ResMut<Assets<StarfieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();

    let layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                ..default()
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        layer,
        Persist,
    ));

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
        layer,
        Persist,
    ));
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

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Default, Clone)]
#[uuid = "1e0463f0-c315-4d84-bf54-f7a1abf93ff5"]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub pos: Vec2,
    #[uniform(0)]
    pub _wasm_padding: Vec2,
}
