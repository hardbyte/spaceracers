mod leaderboard;
mod server_state;

use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::components::ship::Ship;
use crate::game_logic::leaderboard::LeaderBoardPlugin;
use crate::game_state::{GameState, GameStatus};
use crate::{components, game_state};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::{IndexedRandom, SliceRandom};
use rand::Rng;
pub use server_state::ServerState;
use std::time::Duration;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerState>()
            .add_systems(
                Update,
                check_all_players_finished_system.run_if(in_state(ServerState::Active)),
            )
            .add_systems(
                OnEnter(ServerState::Active),
                (setup_scene, spawn_ships, start_game)
                    .before(crate::particle_effects::attach_thruster_effects_to_ships),
            )
            .add_systems(OnExit(ServerState::Active), (unload_game_entities))
            .add_systems(OnEnter(ServerState::Inactive), (setup_game_scheduler))
            .add_systems(
                Update,
                game_scheduler_system.run_if(in_state(ServerState::Inactive)),
            )
            .add_systems(
                PostUpdate,
                transition_to_inactive_system.run_if(has_timer_and_game_finished),
            )
            .add_systems(
                OnExit(ServerState::Active),
                (cleanup_finished_game, cleanup_transition_timer),
            )
            .add_plugins(LeaderBoardPlugin);
    }
}

pub fn start_game(mut commands: Commands, app_state: Res<AppState>) {
    let mut active_game_guard = app_state.active_game.lock().unwrap();
    if let Some(active_game) = active_game_guard.as_mut() {
        active_game.state = game_state::GameStatus::Running;
    } else {
        info!("No active game to start");
    }
}

pub fn unload_game_entities(
    mut commands: Commands,
    query: Query<Entity, With<components::ActiveGameEntity>>,
) {
    // Despawn all entities from the game
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_ships(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
) {
    // Spawn a Ship for each player in the active GameState
    let sprite_size = 25.0;
    let mut rng = rand::thread_rng();

    let active_game_guard = app_state.active_game.lock().unwrap();
    if let Some(active_game) = active_game_guard.as_ref() {
        let sprite_image = asset_server.load(
            active_game
                .map
                .ship_path
                .clone()
                .unwrap_or("ferris.png".to_string())
                .clone(),
        );

        for player in &active_game.players {
            tracing::info!("Adding ship for player {:?}", player.id);
            // Generate a random hue for this player's ship
            let hue = rng.random_range(0.0..360.0);
            let color = Color::hsl(hue, 0.8, 0.5);

            // Pick a random position for the ship from the map's start zones
            let start_region = active_game.map.start_regions.choose(&mut rng).unwrap();

            commands.spawn((
                components::ActiveGameEntity,
                components::ship::ControllableShip {
                    id: player.id,
                    impulse: 8_000.0,
                    torque_impulse: 8_000.0,
                },
                Sprite {
                    //color,
                    image: sprite_image.clone(),
                    custom_size: Some(Vec2::new(sprite_size, sprite_size)),
                    ..Default::default()
                },
                // TODO sample position within the region's polygon
                Transform::from_xyz(start_region.position.x, start_region.position.y, 0.0),
                RigidBody::Dynamic,
                Damping {
                    linear_damping: 0.2,
                    angular_damping: 0.5,
                },
                ExternalImpulse::default(),
                AdditionalMassProperties::Mass(200.0),
                Restitution::coefficient(0.9),
                Friction::coefficient(0.5),
                Collider::ball(sprite_size / 2.0),
                Velocity::default(),
                ActiveEvents::COLLISION_EVENTS,
                ContactForceEventThreshold(10.0),
            ));
        }
    } else {
        info!("No active game to spawn ships for");
    }
}

pub fn setup_scene(
    mut commands: Commands,
    mut rapier_config: Query<&mut RapierConfiguration>,
    app_state: Res<AppState>,
    asset_server: Res<AssetServer>,
) {
    if let Some(active_game) = app_state.active_game.lock().unwrap().as_ref() {
        info!(game_id=?active_game.game_id, "Setting up scene for game");

        let map = &active_game.map;

        // Set up gravity using the map specific value
        rapier_config.single_mut().gravity = Vec2::Y * map.gravity;

        // Obstacles
        for obstacle in &map.obstacles {
            commands.spawn((
                components::ActiveGameEntity,
                Transform::from_xyz(obstacle.position.x, obstacle.position.y, 0.0),
                Collider::polyline(obstacle.polygon.clone(), None),
            ));
        }

        // Finish zone colliders
        // Note we only handle Tiled polygons, ideally handle rectangle objects etc
        for finish in &map.finish_regions {
            commands.spawn((
                components::ActiveGameEntity,
                // Sprite {
                //     image: asset_server.load("finish.png"),
                //     custom_size: Some(Vec2::new(100.0, 75.0)),
                //     ..Default::default()
                // },
                Transform::from_xyz(finish.position.x, finish.position.y, 0.0),
                // Note we only handle Tiled polygons, ideally handle rectangle objects etc
                Collider::polyline(finish.polygon.clone(), None),
                Sensor,
                crate::components::FinishRegion,
            ));
        }

        // Skin
        if let Some(skin_path) = &map.skin_path {
            info!("Spawning background skin from: {}", skin_path);

            // Load the image as a texture
            let texture_handle = asset_server.load(skin_path);

            commands.spawn((
                Sprite {
                    image: texture_handle,
                    custom_size: Some(map.size),
                    ..Default::default()
                },
                // Spawn behind all other entities
                Transform::from_xyz(0.0, 0.0, -100.0),
                // Tag it so we can despawn later if needed
                crate::components::ActiveGameEntity,
            ));
        }
    } else {
        info!("No active game to set up scene for");
        return;
    }
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
                let game_state = GameState::try_from(pending_game.clone())
                    .expect("Failed to create GameState from PendingGame");

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

pub fn cleanup_finished_game(app_state: Res<AppState>) {
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
