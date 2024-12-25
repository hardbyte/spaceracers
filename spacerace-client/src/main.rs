use crossterm::event::{self, Event, KeyCode};
use crossterm::{cursor, execute, terminal};
use rand::distributions::DistString;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{stdout, Write};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info};
use tracing_subscriber;

/// A constant fallback host if the HOST environment variable isn't set.
const DEFAULT_HOST: &str = "http://localhost:5000";

/// A request to join the lobby, registering a player.
#[derive(Debug, Serialize)]
struct LobbyRequest {
    /// The player's name, used for identification.
    name: String,
    /// The player's team name.
    team: String,
    /// The player's password for authentication.
    password: String,
}

/// The response from the server upon successful lobby registration.
#[derive(Debug, Deserialize)]
struct LobbyResponse {
    /// The unique ID of the registered player.
    player_id: String,
    /// The unique game ID that the player is part of.
    game_id: String,
    /// The map name of the current game.
    map: String,
}

/// Represents the response for the current game state.
#[derive(Debug, Deserialize)]
enum GameStateResponse {
    /// State when no active game is in progress.
    Inactive,
    /// State when a game is active. Contains the detailed state.
    Active(ActiveGameState),
}

/// Details of the currently active game state.
#[derive(Debug, Deserialize)]
struct ActiveGameState {
    /// Unique ID of the active game.
    game_id: String,
    /// Current ship states in the game.
    ships: Vec<ShipState>,
    /// Name of the current map.
    map_name: String,
    /// Textual representation of the game state ("Active", "Queued", etc.)
    state: String,
}

/// Represents the state of a single ship.
#[derive(Debug, Deserialize)]
struct ShipState {
    /// Unique ID of this ship.
    id: String,
    /// X, Y position of the ship in the game world.
    position: [f32; 2],
    /// Velocity of the ship in X, Y directions.
    velocity: [f32; 2],
    /// Current orientation (rotation) of the ship.
    orientation: f32,
    /// Angular velocity (rotational speed) of the ship.
    angular_velocity: f32,
}

/// A request to control the ship by applying thrust and rotation.
#[derive(Debug, Serialize)]
struct ControlRequest {
    /// Password for authentication.
    password: String,
    /// Thrust command (1 = forward, -1 = backward, 0 = none).
    thrust: i32,
    /// Rotation command (positive = rotate right, negative = rotate left, 0 = none).
    rotation: i32,
}

/// The server's response to a control request.
#[derive(Debug, Deserialize)]
struct ControlResponse {
    /// Status of the control request. Typically "ok".
    status: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up logging with INFO level by default.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Read configuration from environment variables or use defaults.
    let host = env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let player_name = env::var("PLAYER_NAME").unwrap_or_else(|_| "Player".to_string());
    let player_password = env::var("PLAYER_PASSWORD").unwrap_or_else(|_| {
        rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    });

    let player_team = player_name.clone(); // Use player's name as team name.

    let client = reqwest::Client::new();

    // Register with the lobby
    let lobby_req = LobbyRequest {
        name: player_name.clone(),
        team: player_team.clone(),
        password: player_password.clone(),
    };

    let lobby_response: LobbyResponse = client
        .post(format!("{}/lobby", host))
        .json(&lobby_req)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    info!("Registered with lobby: {:?}", lobby_response);

    // Set up terminal for keyboard input
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide)?;

    // Initial instructions
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    info!("Use arrow keys (or WASD) to control the ship. Press 'q' to quit.");
    stdout.flush()?;

    // Run the game loop
    run_game_loop(client, player_password, host).await?;

    Ok(())
}

/// Run the main game loop:
/// - Periodically fetch the current game state from the server.
/// - Display instructions.
/// - Poll for keyboard input to send control commands.
/// - Sleep briefly to avoid overwhelming the server and CPU.
///
/// # Arguments
///
/// * `client` - The HTTP client for sending requests.
/// * `player_password` - The password of the current player.
/// * `host` - The server host URL.
async fn run_game_loop(
    client: reqwest::Client,
    player_password: String,
    host: String,
) -> anyhow::Result<()> {
    let mut stdout = stdout();

    loop {
        // Get current game state
        let state_response = client
            .get(format!("{}/state", host))
            .send()
            .await?
            .error_for_status()?
            .json::<GameStateResponse>()
            .await?;

        // Clear the screen each loop and move cursor to top-left
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Reprint instructions so they stay visible
        println!("Use arrow keys (or WASD) to control the ship. Press 'q' to quit.");

        // Determine thrust and rotation based on user input
        let thrust_rotation = if let GameStateResponse::Active(active) = &state_response {
            if active.state != "Active" && active.state != "Queued" {
                debug!(
                    "Game is not in an active state (state: {}). Press 'q' to quit.",
                    active.state
                );
                thrust_rotation_from_input_with_timeout(Duration::from_millis(200))?
            } else {
                // Game is active or queued, allow user to control
                thrust_rotation_from_input_with_timeout(Duration::from_millis(200))?
            }
        } else {
            debug!("No active game. Press 'q' to quit.");
            thrust_rotation_from_input_with_timeout(Duration::from_millis(200))?
        };

        // If user provided control input, send it to server
        if let Some((thrust, rotation)) = thrust_rotation {
            let control_req = ControlRequest {
                password: player_password.clone(),
                thrust,
                rotation,
            };

            let control_resp: ControlResponse = client
                .post(format!("{}/control", host))
                .json(&control_req)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            debug!("Control Response: {:?}", control_resp);
        }

        // Output the game state
        if let GameStateResponse::Active(active) = &state_response {
            for ship in &active.ships {
                println!(
                    "Ship ID: {}, Position: ({:.1}, {:.1}), Velocity: ({:.1}, {:.1}), Orientation: {:.1}",
                    ship.id,
                    ship.position[0],
                    ship.position[1],
                    ship.velocity[0],
                    ship.velocity[1],
                    ship.orientation
                );
            }
        }

        stdout.flush()?;
        // Throttle updates slightly to avoid high CPU usage
        sleep(Duration::from_millis(100)).await;
    }
}

/// Polls for keyboard input within the given timeout and returns thrust/rotation commands.
///
/// **Controls:**
/// - Up Arrow or 'w': forward thrust (thrust = 15, rotation = 0)
/// - Down Arrow or 's': backward thrust (thrust = -15, rotation = 0)
/// - Left Arrow or 'a': rotate left (thrust = 0, rotation = 15)
/// - Right Arrow or 'd': rotate right (thrust = 0, rotation = -15)
/// - 'q': quit the application
///
/// If no input is provided within the timeout, returns `None`.
///
/// # Arguments
///
/// * `timeout` - Duration to wait for input before timing out.
fn thrust_rotation_from_input_with_timeout(
    timeout: Duration,
) -> anyhow::Result<Option<(i32, i32)>> {
    let mut thrust = 0;
    let mut rotation = 0;

    // Poll once for events within the given timeout.
    if event::poll(timeout)? {
        // Keep reading events that have arrived simultaneously
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('w') => thrust = 15,
                    KeyCode::Down | KeyCode::Char('s') => thrust = -15,
                    KeyCode::Left | KeyCode::Char('a') => rotation = 15,
                    KeyCode::Right | KeyCode::Char('d') => rotation = -15,
                    KeyCode::Char('q') => {
                        cleanup_terminal()?;
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }

        // If we captured any thrust/rotation inputs, return them.
        if thrust != 0 || rotation != 0 {
            return Ok(Some((thrust, rotation)));
        } else {
            return Ok(None);
        }
    } else {
        Ok(None)
    }
}

/// Cleans up the terminal by disabling raw mode and showing the cursor again.
///
/// Call this before exiting to restore the user's terminal state to normal.
fn cleanup_terminal() -> anyhow::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    execute!(std::io::stdout(), cursor::Show)?;
    Ok(())
}
