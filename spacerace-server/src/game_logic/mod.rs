use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::components::ship::Ship;
use crate::game_state::{GameState, GameStatus};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            transition_to_inactive_system.run_if(has_timer_and_game_finished),
        )
        .add_systems(
            OnExit(ServerState::Active),
            (cleanup_finished_game, cleanup_transition_timer),
        );
    }
}

// Enum that will be used as a global state for the game server
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ServerState {
    #[default]
    Inactive,
    Active,
}

// System to check if all players finished the race
pub fn check_all_players_finished_system(app_state: Res<AppState>, mut commands: Commands) {
    let mut active_game_lock = app_state.active_game.lock().unwrap();
    if let Some(active_game) = active_game_lock.as_mut() {
        if active_game.state == GameStatus::Running
            && active_game.players.len() == active_game.finish_times.len()
        {
            info!("All players have finished the race! Transitioning game state to Finished.");
            active_game.state = GameStatus::Finished;

            // Transition to Inactive after a delay
            setup_transition_timer(commands);
        }
    }
}

#[derive(Resource)]
pub struct GameSchedulerConfig {
    pub timer: Timer,
}

pub fn setup_game_scheduler(mut commands: Commands) {
    tracing::info!("Setting up game scheduler");
    commands.insert_resource(GameSchedulerConfig {
        timer: Timer::from_seconds(10.0, TimerMode::Repeating),
    });
}

#[tracing::instrument(skip_all)]
pub fn game_scheduler_system(
    app_state: Res<AppState>,
    time: Res<Time>,
    mut next_server_state: ResMut<NextState<ServerState>>,
    mut config: ResMut<GameSchedulerConfig>,
) {
    // tick the timer
    config.timer.tick(time.delta());

    if config.timer.finished() {
        tracing::info!("Game scheduler system running");

        let mut lobby = app_state.lobby.lock().unwrap();
        let mut active_game = app_state.active_game.lock().unwrap();

        // If there's no active game, check if we can start one
        if active_game.is_none() {
            tracing::debug!("No active game, checking lobby");
            if let Some(index) = lobby.iter().position(|game| !game.players.is_empty()) {
                tracing::info!(game_id=?lobby[index].game_id, "Promoting game from lobby to active");
                // Remove the pending game from the lobby
                let pending_game = lobby.remove(index);

                // Create a new GameState from the pending game
                let game_state = GameState::from(pending_game.clone());

                tracing::info!(game.id=?game_state.game_id, state=?game_state, "Starting game");

                // Update the active game
                *active_game = Some(game_state.clone());

                tracing::info!("Transitioning Server State to Active");

                // Transition to Active
                next_server_state.set(ServerState::Active);
            }
        } else {
            tracing::debug!("Active game already exists");
        }
    }
}

#[derive(Resource)]
struct TransitionTimer(Timer);

fn setup_transition_timer(mut commands: Commands) {
    // Add a 10-second timer
    commands.insert_resource(TransitionTimer(Timer::new(
        Duration::from_secs(10),
        TimerMode::Once,
    )));
}

fn transition_to_inactive_system(
    mut timer: ResMut<TransitionTimer>,
    mut state: ResMut<NextState<ServerState>>,
    time: Res<Time>,
) {
    // Update the timer and transition to Inactive if completed
    if timer.0.tick(time.delta()).finished() {
        info!("Transitioning Server State to Inactive");
        state.set(ServerState::Inactive);
    }
}

// Runs if the `TransitionTimer` exists and the game is finished
fn has_timer_and_game_finished(
    app_state: Res<AppState>,
    timer: Option<Res<TransitionTimer>>,
) -> bool {
    timer.is_some()
        && app_state
            .active_game
            .lock()
            .unwrap()
            .as_ref()
            .map_or(false, |game| matches!(game.state, GameStatus::Finished))
}

pub fn cleanup_finished_game(app_state: Res<AppState>, mut commands: Commands) {
    // Remove the active game
    let mut active_game = app_state.active_game.lock().unwrap();

    if let Some(game) = active_game.as_ref() {
        info!(game.id=?game.game_id, "Cleaning up finished game");
        active_game.take();
    } else {
        info!("No active game to clean up");
    }
}

pub fn cleanup_transition_timer(mut commands: Commands) {
    commands.remove_resource::<TransitionTimer>();
}
