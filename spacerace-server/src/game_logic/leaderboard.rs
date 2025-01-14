use bevy::app::Plugin;
use bevy::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use crate::app_state::AppState;

use crate::game_logic::server_state::ServerState;

/// Main plugin struct.
pub struct LeaderBoardPlugin;

/// Marker component for the top-level leaderboard UI node.
#[derive(Component)]
struct LeaderboardUIRoot;

/// Marker component for an individual player's score line.
#[derive(Component)]
struct PlayerScoreUI {
    player_id: Uuid,
}

/// Resource holding the entities related to the leaderboard.
/// Storing these allows us to clean up or update them later.
#[derive(Resource, Default)]
struct LeaderboardUIState {
    /// Map of `player_id` -> UI entity for that player's line.
    score_entities: HashMap<Uuid, Entity>,
}

impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize our UI state resource.
            .init_resource::<LeaderboardUIState>()
            // When entering the Active state, spawn the leaderboard UI.
            .add_systems(OnEnter(ServerState::Active), setup_leaderboard_ui)
            // When exiting the Active state, despawn all leaderboard UI.
            .add_systems(OnExit(ServerState::Active), cleanup_leaderboard_ui)
            // Continuously update the leaderboard UI while in Active state.
            .add_systems(
                Update,
                update_leaderboard_ui_system.run_if(in_state(ServerState::Active)),
            );
    }
}

/// Spawns the top-level leaderboard UI node.
/// This node will contain the "Leaderboard" text header and the subsequent player lines.
fn setup_leaderboard_ui(mut commands: Commands) {
    info!("Setting up the leaderboard UI.");

    // Spawn a root node to hold the leaderboard.
    // We give it a semi-transparent black background using an alpha channel < 1.0.
    let leaderboard_root = commands
        .spawn((
            // A Node with flexible dimensions and layout.
            Node {
                // We want an absolutely positioned UI node
                position_type: PositionType::Absolute,
                // Anchor to the right of the screen.
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                // Give it a fixed width, and full vertical height
                width: Val::Px(200.0),
                height: Val::Percent(100.0),

                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            // Semi-transparent background color.
            BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.3)),
            LeaderboardUIRoot,
        ))
        .with_children(|parent| {
            // A simple text node for our leaderboard title.
            parent.spawn((
                Text::new("Leaderboard"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            ));
        })
        .id();

    debug!("Spawned leaderboard root entity: {:?}", leaderboard_root);
}

/// Removes the entire leaderboard UI tree when leaving the Active state.
fn cleanup_leaderboard_ui(
    mut commands: Commands,
    leaderboard_root_query: Query<Entity, With<LeaderboardUIRoot>>,
    mut leaderboard_ui_state: ResMut<LeaderboardUIState>,
) {
    info!("Cleaning up the leaderboard UI.");

    // Despawn the root node and all its children
    for entity in leaderboard_root_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Clear our stored UI entity references
    leaderboard_ui_state.score_entities.clear();
}

/// Continuously updates the leaderboard UI based on current game info.
/// - Sorts players by finish times.
/// - Spawns/cleans up UI for each player as needed.
fn update_leaderboard_ui_system(
    mut commands: Commands,
    app_state: Res<AppState>,
    mut leaderboard_ui_state: ResMut<LeaderboardUIState>,
    leaderboard_root_query: Query<Entity, With<LeaderboardUIRoot>>,
    existing_entities: Query<(), With<Parent>>,
) {
    let leaderboard_root = match leaderboard_root_query.get_single() {
        Ok(e) => e,
        Err(_) => {
            debug!("No LeaderboardUIRoot found; skipping update.");
            return;
        }
    };
    // Get the active game state
    let active_game = app_state.active_game.lock().unwrap();
    let Some(game) = active_game.as_ref() else {
        debug!("No active game found; skipping leaderboard update.");
        return;
    };

    // Sort players by their finish time, name
    let mut players = game.players.clone();
    players.sort_by(|a, b| {
        game.finish_times
            .get(&a.id)
            .unwrap_or(&f32::MAX)
            .partial_cmp(game.finish_times.get(&b.id).unwrap_or(&f32::MAX))
            .unwrap()
            .then_with(|| a.name.cmp(&b.name))
    });

    // Collect all the player IDs we have in the game
    let mut seen_ids = Vec::with_capacity(players.len());

    for (i, player) in players.iter().enumerate() {
        let rank = i + 1;
        seen_ids.push(player.id);

        // Check if we already have a UI entity for this player
        let line_entity = leaderboard_ui_state.score_entities.get(&player.id).cloned();

        // If not, spawn it
        let line_entity = if let Some(entity) = line_entity {
            entity
        } else {
            debug!("Spawning new player line for: {}", player.name);
            let new_entity = commands
                .entity(leaderboard_root)
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            // Each line is a row
                            flex_direction: FlexDirection::Row,
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        PlayerScoreUI {
                            player_id: player.id,
                        },
                    ));
                })
                .id();
            leaderboard_ui_state
                .score_entities
                .insert(player.id, new_entity);
            new_entity
        };

        // Clear out old text children
        commands.entity(line_entity).despawn_descendants();

        // Format the rank and time
        let finish_time = game
            .finish_times
            .get(&player.id)
            .map_or("In Progress".to_string(), |&time| format!("{:.2}", time));

        // Put a line of text: "Rank. player_name - finish_time"
        commands.entity(line_entity).with_children(|parent| {
            parent.spawn((
                Text::new(format!("{}: {} - {}", rank, player.name, finish_time)),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
            ));
        });
    }

    // Despawn UI for players no longer in the game
    let players_to_remove: Vec<Uuid> = leaderboard_ui_state
        .score_entities
        .keys()
        .copied()
        .filter(|pid| !seen_ids.contains(pid))
        .collect();

    for pid in players_to_remove {
        if let Some(entity) = leaderboard_ui_state.score_entities.remove(&pid) {
            if existing_entities.contains(entity) {
                debug!("Removing player line for old/removed player: {:?}", pid);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
