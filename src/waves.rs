use bevy::prelude::*;

use crate::{enemy::SpawnEnemyEvent, GameState};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        let mut waves = Waves::default();
        waves.waves.push(Wave {
            delay: 10.,
            num: 4,
            interval: 4.,
            hp: 4,
        });
        waves.waves.push(Wave {
            delay: 30.,
            num: 8,
            interval: 4.,
            hp: 4,
        });
        waves.waves.push(Wave {
            delay: 30.,
            num: 4,
            interval: 4.,
            hp: 10,
        });
        app.insert_resource(WaveState::from(&waves.waves[0]))
            .insert_resource(waves);

        app.add_system(spawn_enemies.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Resource, Default)]
pub struct Waves {
    pub waves: Vec<Wave>,
    pub current: usize,
}
impl Waves {
    pub fn current(&self) -> Option<&Wave> {
        self.waves.get(self.current)
    }
    pub fn advance(&mut self) -> Option<&Wave> {
        self.current += 1;
        self.current()
    }
}

#[derive(Clone, Debug)]
pub struct Wave {
    pub num: usize,
    pub hp: u32,
    pub interval: f32,
    pub delay: f32,
}

#[derive(Resource)]
pub struct WaveState {
    pub delay_timer: Timer,
    pub spawn_timer: Timer,
    pub remaining: usize,
}

impl From<&Wave> for WaveState {
    fn from(value: &Wave) -> Self {
        Self {
            delay_timer: Timer::from_seconds(value.delay, TimerMode::Once),
            spawn_timer: Timer::from_seconds(value.interval, TimerMode::Repeating),
            remaining: value.num,
        }
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut waves: ResMut<Waves>,
    mut wave_state: ResMut<WaveState>,
    time: Res<Time>,
    mut events: EventWriter<SpawnEnemyEvent>,
) {
    let Some(current_wave) = waves.current() else {
        return;
    };

    wave_state.delay_timer.tick(time.delta());
    if !wave_state.delay_timer.finished() {
        return;
    }

    wave_state.spawn_timer.tick(time.delta());
    if !wave_state.spawn_timer.just_finished() {
        return;
    }

    events.send(SpawnEnemyEvent {
        hp: current_wave.hp,
    });

    wave_state.remaining -= 1;

    if wave_state.remaining == 0 {
        if let Some(next) = waves.advance() {
            commands.insert_resource(WaveState::from(next));
        }
    }
}
