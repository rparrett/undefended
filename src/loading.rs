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
    #[asset(path = "models/towerbase.glb#Scene0")]
    pub tower_base: Handle<Scene>,
    #[asset(path = "models/player.glb#Scene0")]
    pub player: Handle<Scene>,
}

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, Models>(GameState::Loading);
    }
}
