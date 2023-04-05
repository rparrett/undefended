use bevy::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_loading.in_schedule(OnEnter(GameState::Loading)));
    }
}

fn start_loading(mut next_state: ResMut<NextState<GameState>>) {
    // TODO actually load some stuff
    next_state.set(GameState::Playing);
}
