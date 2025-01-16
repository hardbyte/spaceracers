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

#[derive(Component)]
struct PlayerScoreText {
    player_id: Uuid,
}

/// Resource holding the entities related to the leaderboard.
#[derive(Resource, Default)]
struct LeaderboardUIState {
    /// Map of `player_id` -> Node entity
    line_entities: HashMap<Uuid, Entity>,
    /// Map of `player_id` -> the text entity for that Node
    text_entities: HashMap<Uuid, Entity>,
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
    let _leaderboard_root = commands
        .spawn((
            // A Node with flexible dimensions and layout.
            Node {
                // We want an absolutely positioned UI node
                position_type: PositionType::Absolute,
                // Anchor to the right of the screen.
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(200.0),
                height: Val::Percent(100.0),

                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            // Semi-transparent background color.
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            LeaderboardUIRoot,
        ))
        .with_children(|parent| {
            // A simple header node for our leaderboard
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|leaderboard_node| {
                    leaderboard_node.spawn((
                        Text::new("Spaceracers"),
                        TextColor(Color::BLACK),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                    ));
                });
        });
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
    leaderboard_ui_state.line_entities.clear();
    leaderboard_ui_state.text_entities.clear();
}

/// Continuously updates the leaderboard UI based on current game info.
fn update_leaderboard_ui_system(
    mut commands: Commands,
    app_state: Res<AppState>,
    mut leaderboard_ui_state: ResMut<LeaderboardUIState>,
    leaderboard_root_query: Query<Entity, With<LeaderboardUIRoot>>,
    game_time: Res<Time>,
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

    let mut seen_ids = Vec::with_capacity(players.len());
    // store the line entities in sorted order for re-insertion.
    let mut sorted_line_entities = Vec::with_capacity(players.len());

    for (i, player) in players.iter().enumerate() {
        seen_ids.push(player.id);

        // Check if we already have a line entity for this player
        let line_entity = leaderboard_ui_state.line_entities.get(&player.id).copied();
        let text_entity = leaderboard_ui_state.text_entities.get(&player.id).copied();

        // If there's no line entity for this player, spawn one
        let line_entity = match line_entity {
            Some(entity) => entity,
            None => {
                // Create the Node for the line
                let new_line_entity = commands
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            margin: UiRect::all(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        PlayerScoreUI {
                            player_id: player.id,
                        },
                    ))
                    // Next, spawn a single child text entity.
                    .with_children(|line_parent| {
                        let new_text_entity = line_parent
                            .spawn((
                                Text::new(""),
                                TextColor(Color::BLACK),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                PlayerScoreText {
                                    player_id: player.id,
                                },
                            ))
                            .id();

                        // Record that text entity in our resource so we can update it
                        leaderboard_ui_state
                            .text_entities
                            .insert(player.id, new_text_entity);
                    })
                    .id();

                // Attach the new line entity to the leaderboard root
                commands.entity(leaderboard_root).add_child(new_line_entity);

                // Store the line entity in our resource
                leaderboard_ui_state
                    .line_entities
                    .insert(player.id, new_line_entity);

                new_line_entity
            }
        };

        // Now that we have a line entity, we **should** have a text entity for it too.
        let text_entity = match text_entity {
            Some(e) => e,
            None => {
                // Shouldn’t usually happen, but we can handle it gracefully.
                let fallback_text_entity = commands
                    .entity(line_entity)
                    .with_children(|line_parent| {
                        let new_text_entity = line_parent
                            .spawn((
                                Text::new(""),
                                TextColor(Color::BLACK),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                PlayerScoreText {
                                    player_id: player.id,
                                },
                            ))
                            .id();
                        leaderboard_ui_state
                            .text_entities
                            .insert(player.id, new_text_entity);
                    })
                    .id();
                fallback_text_entity
            }
        };

        // Format the rank and time
        let optional_finish_time = game.finish_times.get(&player.id);
        let finish_time = optional_finish_time
            .map_or(format!("{:.2}", game_time.elapsed_secs()), |&time| {
                format!("{:.2}", time)
            });

        let rank = match optional_finish_time {
            Some(_) => format!("{}.", i + 1),
            None => "-".to_string(),
        };
        let score_text = format!("{}: {} {}", rank, player.name, finish_time);

        // Update the text in place by inserting a new Text component
        // or by using an entity command with `.insert()`.
        commands.entity(text_entity).insert(Text::new(score_text));

        // Keep track of this line in our sorted list
        sorted_line_entities.push(line_entity);
    }

    // Despawn UI for players no longer in the game
    let players_to_remove: Vec<Uuid> = leaderboard_ui_state
        .line_entities
        .keys()
        .copied()
        .filter(|pid| !seen_ids.contains(pid))
        .collect();

    for pid in players_to_remove {
        if let Some(line_entity) = leaderboard_ui_state.line_entities.remove(&pid) {
            info!("Removing player line for old/removed player: {:?}", pid);
            commands.entity(line_entity).despawn_recursive();
        }
        // Also remove the text entity reference
        leaderboard_ui_state.text_entities.remove(&pid);
    }

    // Reorder based on sorted_line_entities
    // First, remove all children from the leaderboard root (this doesn't despawn them).
    commands.entity(leaderboard_root).clear_children();

    // Push them back in the order we want from top to bottom
    commands
        .entity(leaderboard_root)
        .add_children(&sorted_line_entities);
}
