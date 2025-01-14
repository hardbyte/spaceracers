use bevy::app::{App, Plugin, PluginGroup};

use bevy::color::Color;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use std::collections::HashMap;

use crate::app_state::AppState;
use crate::game_logic::ServerState;
use crate::game_state::PendingGame;
use uuid::Uuid;

pub struct LobbyGraphicsPlugin;

#[derive(Resource, Default)]
pub struct LobbyUIState {
    /// Map of `game_id` -> Entity for the top-level UI node of that game.
    pub game_nodes: HashMap<Uuid, Entity>,
    /// Map of `(game_id, player_id)` -> Entity for the player UI node.
    pub player_nodes: HashMap<(Uuid, Uuid), Entity>,
}

impl Plugin for LobbyGraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyUIState>()
            //.insert_resource(WinitSettings::desktop_app())
            .add_systems(OnEnter(ServerState::Inactive), setup_lobby_ui)
            .add_systems(OnExit(ServerState::Inactive), unload_lobby_ui)
            .add_systems(
                Update,
                update_lobby_ui_system.run_if(in_state(ServerState::Inactive)),
            )
            .add_systems(
                Update,
                button_interaction_system.run_if(in_state(ServerState::Inactive)),
            );
    }
}

#[derive(Component)]
struct LobbyUIRoot;

/// Marker for the dynamic part of the UI that we replace every frame
#[derive(Component)]
struct LobbyGamesContainer;

#[derive(Component)]
struct GameUI {
    game_id: Uuid,
}

#[derive(Component)]
struct PlayerUI {
    player_id: Uuid,
}

#[derive(Component)]
struct QuitGameButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.95, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.95, 0.35);

pub fn setup_lobby_ui(mut commands: Commands) {
    // Root UI node that fills the entire screen
    commands
        .spawn((
            // Use a `Node` with Flexbox layout
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK), // entire screen background
            LobbyUIRoot,
        ))
        .with_children(|parent| {
            // Title
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|text_node| {
                    text_node.spawn((
                        Text::new("SpacerRacers"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                    ));
                    text_node.spawn((
                        Text::new("Waiting for players to join..."),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));
                });

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                LobbyGamesContainer,
            ));

            // Quit button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    QuitGameButton,
                ))
                .with_children(|btn| {
                    btn.spawn((Text::new("Quit Game"), TextColor(Color::WHITE)));
                });
        });
}
pub fn unload_lobby_ui(mut commands: Commands, lobby_query: Query<Entity, With<LobbyUIRoot>>) {
    for entity in lobby_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Periodically updates our lobby UI
/// - Shows a list of pending games and their players
pub fn update_lobby_ui_system(
    mut commands: Commands,
    app_state: Res<AppState>,
    mut lobby_ui_state: ResMut<LobbyUIState>,
    // All game nodes query
    container_query: Query<Entity, With<LobbyGamesContainer>>,
    existing_entities: Query<(), With<Parent>>,
) {
    let container_entity = match container_query.get_single() {
        Ok(e) => e,
        Err(_) => return, // No container, means we are not in the Lobby state
    };

    // Snapshot of pending games
    let pending_games = app_state.lobby.lock().unwrap().clone();
    let mut seen_game_ids = Vec::new();
    for game in pending_games.iter() {
        seen_game_ids.push(game.game_id);

        let game_entity = if let Some(&existing) = lobby_ui_state.game_nodes.get(&game.game_id) {
            // Already exists
            existing
        } else {
            // Spawn the game UI node
            let new_entity = commands
                .entity(container_entity)
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                margin: UiRect::all(Val::Px(10.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            GameUI {
                                game_id: game.game_id,
                            },
                        ))
                        .with_children(|game_node| {
                            // Display info about the game
                            game_node.spawn((
                                Text::new(format!(
                                    "Game: {} | Map: {}",
                                    game.game_id, game.map_name
                                )),
                                TextColor(Color::WHITE),
                            ));
                        });
                })
                .id();
            lobby_ui_state.game_nodes.insert(game.game_id, new_entity);
            new_entity
        };

        // Ensure the *players* for this game are up to date
        update_game_players(&mut commands, &mut lobby_ui_state, game_entity, game);
    }

    // 2) Despawn UI for any game that no longer appears in `pending_games`
    let game_ids_to_remove: Vec<Uuid> = lobby_ui_state
        .game_nodes
        .keys()
        .filter(|game_id| !seen_game_ids.contains(game_id))
        .copied()
        .collect();

    for old_game_id in game_ids_to_remove {
        if let Some(entity) = lobby_ui_state.game_nodes.remove(&old_game_id) {
            // Also remove any player nodes for that game
            lobby_ui_state
                .player_nodes
                .retain(|(gid, _), player_entity| {
                    if *gid == old_game_id {
                        // Despawn it
                        if existing_entities.contains(*player_entity) {
                            commands.entity(*player_entity).despawn_recursive();
                        }
                        false
                    } else {
                        true
                    }
                });

            // Finally, remove the game node itself
            if existing_entities.contains(entity) {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn update_game_players(
    commands: &mut Commands,
    lobby_ui_state: &mut LobbyUIState,
    game_entity: Entity,
    pending_game: &PendingGame,
) {
    // Build a set of all current players in the pending game
    let mut seen_player_ids = Vec::with_capacity(pending_game.players.len());

    for player in &pending_game.players {
        seen_player_ids.push(player.id);

        // We use a composite key (game_id, player_id)
        let key = (pending_game.game_id, player.id);
        lobby_ui_state.player_nodes.entry(key).or_insert_with(|| {
            // Create a new UI node for this player
            commands
                .entity(game_entity)
                .with_children(|game_node| {
                    game_node
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::left(Val::Px(10.0)),
                                ..default()
                            },
                            PlayerUI {
                                player_id: player.id,
                            },
                        ))
                        .with_children(|player_node| {
                            let team_text = player
                                .team
                                .as_ref()
                                .map(|t| format!(" (Team: {t})"))
                                .unwrap_or_else(|| "(No team)".to_string());

                            player_node.spawn((
                                Text::new(format!("{}{}", player.name, team_text)),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                            ));
                        });
                })
                .id()
        });
    }

    // Remove any UI nodes for players who aren’t in this `pending_game` anymore
    let game_id = pending_game.game_id;
    let player_ids_to_remove: Vec<(Uuid, Uuid)> = lobby_ui_state
        .player_nodes
        .keys()
        .filter(|(gid, _)| *gid == game_id)
        .filter(|(_, pid)| !seen_player_ids.contains(pid))
        .copied()
        .collect();

    for key_to_remove in player_ids_to_remove {
        if let Some(entity) = lobby_ui_state.player_nodes.remove(&key_to_remove) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// A system to handle button state changes and clicks.
/// Updates the button's background color and border color, and triggers logic on click.
fn button_interaction_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>, With<QuitGameButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in &mut query {
        tracing::info!("Button interaction: {:?}", interaction);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
                // TODO Use an event to trigger the start
            }
            Interaction::Hovered => {
                *bg_color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *bg_color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
