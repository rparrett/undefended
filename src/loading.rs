use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

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
}

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MainMenu),
        )
        .add_collection_to_loading_state::<_, Models>(GameState::Loading)
        .add_collection_to_loading_state::<_, Fonts>(GameState::Loading)
        .add_collection_to_loading_state::<_, Sounds>(GameState::Loading);
    }
}
