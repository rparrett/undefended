use bevy::{
    prelude::*,
    render::{
        render_resource::{CachedPipelineState, PipelineCache},
        Render, RenderApp, RenderSet,
    },
};
use bevy_asset_loader::prelude::*;
use crossbeam_channel::Receiver;

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

#[derive(Resource)]

struct PipelineStatus(Receiver<bool>);

const EXPECTED_PIPELINES: usize = 10;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::bounded(1);

        app.insert_resource(PipelineStatus(rx));

        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Pipelines),
        )
        .add_collection_to_loading_state::<_, Models>(GameState::Loading)
        .add_collection_to_loading_state::<_, Fonts>(GameState::Loading)
        .add_collection_to_loading_state::<_, Images>(GameState::Loading)
        .add_collection_to_loading_state::<_, Sounds>(GameState::Loading)
        .add_systems(
            Update,
            pipelines_done.run_if(in_state(GameState::Pipelines)),
        )
        .add_systems(OnExit(GameState::Pipelines), cleanup)
        .add_systems(OnEnter(GameState::Pipelines), setup_pipelines);

        let renderer_app = app.sub_app_mut(RenderApp);
        let mut done = false;
        renderer_app.add_systems(
            Render,
            (move |cache: Res<PipelineCache>| {
                if done {
                    return;
                }

                let ready = cache
                    .pipelines()
                    .filter(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_)))
                    .count();

                debug!("pipelines ready: {}/{}", ready, EXPECTED_PIPELINES);

                if ready >= EXPECTED_PIPELINES {
                    let _ = tx.send(true);
                    done = true
                }
            })
            .in_set(RenderSet::Cleanup),
        );
    }
}

fn setup_pipelines(mut commands: Commands, models: Res<Models>) {
    commands.spawn((
        PipelinesMarker,
        SceneBundle {
            scene: models.player.clone(),
            ..default()
        },
    ));
}

fn pipelines_done(status: Res<PipelineStatus>, mut next_state: ResMut<NextState<GameState>>) {
    if status.0.try_recv().unwrap_or_default() {
        next_state.set(GameState::MainMenu);
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PipelinesMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
