use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};

use crate::GameState;

pub struct LoadingPlugin;

#[derive(Component)]
pub struct PipelinesMarker;

#[derive(AssetCollection, Resource)]
pub struct Models {
    #[asset(path = "models/tile1.glb#Scene0")]
    pub tile1: Handle<Scene>,
    #[asset(path = "models/tile2.glb#Scene0")]
    pub tile2: Handle<Scene>,
    #[asset(path = "models/tile3.glb#Scene0")]
    pub tile3: Handle<Scene>,
    #[asset(path = "models/tile4.glb#Scene0")]
    pub tile4: Handle<Scene>,
    #[asset(path = "models/itemspawner.glb#Scene0")]
    pub item_spawner: Handle<Scene>,
    #[asset(path = "models/towerbase.glb#Scene0")]
    pub tower_base: Handle<Scene>,
    #[asset(path = "models/towerheadsm.glb#Scene0")]
    pub tower_head: Handle<Scene>,
    #[asset(path = "models/towerkit.glb#Scene0")]
    pub tower_kit: Handle<Scene>,
    #[asset(path = "models/laserammo.glb#Scene0")]
    pub laser_ammo: Handle<Scene>,
    #[asset(path = "models/enemy1.glb#Scene0")]
    pub enemy1: Handle<Scene>,
    #[asset(path = "models/player.glb#Scene0")]
    pub player: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "fonts/Orbitron-Medium.ttf")]
    pub main: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct Sounds {
    #[asset(path = "sounds/music.ogg")]
    pub music: Handle<AudioSource>,
    #[asset(path = "sounds/build.ogg")]
    pub build: Handle<AudioSource>,
    #[asset(path = "sounds/bad.ogg")]
    pub bad: Handle<AudioSource>,
    #[asset(path = "sounds/feed.ogg")]
    pub feed: Handle<AudioSource>,
    #[asset(path = "sounds/powerdown.ogg")]
    pub powerdown: Handle<AudioSource>,
    #[asset(path = "sounds/damage.ogg")]
    pub damage: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct Images {
    #[asset(path = "images/heart.png")]
    pub heart: Handle<Image>,
}

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 15;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 13;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PipelinesReadyPlugin)
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .load_collection::<Models>()
                    .load_collection::<Fonts>()
                    .load_collection::<Images>()
                    .load_collection::<Sounds>()
                    .continue_to_state(GameState::Pipelines),
            )
            .add_systems(
                Update,
                pipelines_done.run_if(in_state(GameState::Pipelines)),
            )
            .add_systems(OnExit(GameState::Pipelines), cleanup)
            .add_systems(OnEnter(GameState::Pipelines), setup_pipelines);
    }
}

fn setup_pipelines(
    mut commands: Commands,
    models: Res<Models>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        PipelinesMarker,
        TextBundle::from_section("Loading Pipelines...".to_string(), TextStyle::default()),
    ));

    // Spawn enough things to trigger the creation of all the pipelines required for the
    // game.

    commands.spawn((
        PipelinesMarker,
        SceneBundle {
            scene: models.player.clone(),
            ..default()
        },
    ));

    let path_mat = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        PipelinesMarker,
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(0.25, 0.25, 0.25))),
            material: path_mat.clone(),
            ..default()
        },
    ));

    commands.spawn((
        PipelinesMarker,
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 2500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -1.0, -1.0, -1.0)),
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 30.0,
                ..default()
            }
            .into(),
            ..default()
        },
    ));
}

fn pipelines_done(ready: Res<PipelinesReady>, mut next_state: ResMut<NextState<GameState>>) {
    info!("Pipelines Ready: {}/{}", ready.get(), EXPECTED_PIPELINES);

    if ready.get() >= EXPECTED_PIPELINES {
        next_state.set(GameState::MainMenu);
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PipelinesMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
