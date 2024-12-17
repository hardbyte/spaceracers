use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::game_state::{GameState, GameStatus};
use crate::ship::Ship;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPRITE_SIZE: f32 = 25.0;

// Enum that will be used as a global state for the game server
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ServerState {
    #[default]
    Inactive,
    Active,
}

#[derive(Debug, Event)]
pub enum GameEvent {
    GameStarted { game_id: uuid::Uuid },
    GameFinished { game_id: uuid::Uuid },
}

// This system will run during the Active state
#[tracing::instrument(skip_all)]
pub fn game_system(
    mut current_server_state: ResMut<State<ServerState>>,
    mut next_server_state: ResMut<NextState<ServerState>>,
    app_state: Res<AppState>,
) {
    tracing::trace!("In game system running");

    // TODO we may not need this at all...
    // perhaps need to call periodically and check if the game is over

    // If the game is over we can either transition to Inactive or directly start a new game from the lobby
    //
    // let lobby = app_state.lobby.lock().unwrap();
    // if current_server_state.get() == &ServerState::Active && !lobby.is_empty() {
    //     tracing::info!("Transitioning directly to Active state");
    //     // Transition to Active
    //     next_server_state.set(ServerState::Active);
    // } else if current_server_state.get() == &ServerState::Active {
    //     tracing::info!("Transitioning to Inactive state");
    //     // Transition to Inactive
    //     next_server_state.set(ServerState::Inactive);
    // } else {
    //     // Remain in Inactive state
    //     next_server_state.set(ServerState::Inactive);
    // }
}

#[derive(Resource)]
pub struct GameSchedulerConfig {
    pub timer: Timer,
    //pub game_start_delay: Duration,
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
    //mut writer: EventWriter<GameEvent>
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

                // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/event.rs
                // tracing::debug!("Sending a GameStarted event");
                // writer.send(GameEvent::GameStarted {
                //     game_id: game_state.game_id,
                // });

                tracing::info!("Transitioning Server State to Active");
                // Transition to Active
                next_server_state.set(ServerState::Active);
            }
        }
    }
}

fn get_starting_positions(num_players: usize) -> Vec<Vec2> {
    // Return a list of starting positions based on the number of players and the current map
    // For simplicity, we'll hardcode some positions
    vec![
        Vec2::new(-200.0, 150.0),
        Vec2::new(-200.0, 100.0),
        Vec2::new(-200.0, 50.0),
        Vec2::new(-200.0, 0.0),
    ]
}
