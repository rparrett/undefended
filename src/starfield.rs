use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE,
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PrimitiveState, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::{GameState, Persist, Player};

// This attribute only exists to ensure that our starfield mesh ends up in its own
// unique vertex buffer, which allows the built-in `FULLSCREEN_SHADER_HANDLE` vertex
// shader to receive correct "local vertex indices" and this to function properly.
//
// Yes, this is sort of a ridiculous amount of effort to avoid using a custom
// pipeline and vertex shader.
const ATTRIBUTE_STARFIELD: MeshVertexAttribute =
    MeshVertexAttribute::new("StarField", 0xbbbcb1d9d46bac97, VertexFormat::Float32);

pub struct StarfieldPlugin;
#[derive(Component)]
struct Starfield;

#[derive(Asset, AsBindGroup, TypePath, Debug, Default, Clone)]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub pos: Vec2,
    #[uniform(0)]
    pub _wasm_padding: Vec2,
}

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
) {
    let layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        layer.clone(),
        Persist,
    ));

    commands.spawn((
        Mesh2d(meshes.add(starfield_mesh())),
        MeshMaterial2d(mat2d.add(StarfieldMaterial::default())),
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
            mat.1.pos = player.translation.truncate()
        }
    }
}

fn starfield_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
    // Bevy will panic if our mesh doesn't have positions. These get rewritten by
    // the vertex shader.
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3])
    .with_inserted_attribute(ATTRIBUTE_STARFIELD, vec![0.0; 3])
}

impl Material2d for StarfieldMaterial {
    fn vertex_shader() -> ShaderRef {
        FULLSCREEN_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive = PrimitiveState::default();
        descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
        Ok(())
    }
}
