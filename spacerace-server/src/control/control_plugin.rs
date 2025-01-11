use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::game_logic::ServerState;
use crate::components::ship::Ship;
use crate::{components, game_logic, setup_scene};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{ExternalImpulse, Velocity};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ServerState::Active), setup_controls);
        app.add_systems(OnExit(ServerState::Active), cleanup_controls);
        app.add_systems(
            Update,
            apply_controls_system.run_if(in_state(ServerState::Active)),
        );
        app.add_systems(
            PostUpdate,
            update_public_game_state_system.run_if(in_state(ServerState::Active)),
        );
    }
}

fn setup_controls(app_state: Res<AppState>) {
    // TODO!
    // Create a new control input for each ship in the current game
    // Add it to `AppState.control_inputs`
}

fn cleanup_controls(app_state: Res<AppState>) {
    // TODO!
    // Remove all control inputs for the current game
    // Remove them from `AppState.control_inputs`
}

fn apply_controls_system(
    app_state: Res<AppState>,
    mut player_info: Query<(
        &components::ship::ControllableShip,
        &mut Transform,
        &mut ExternalImpulse,
    )>,
) {
    let mut control_inputs_lock = app_state.control_inputs.lock().unwrap();

    // Check `AppState.control_inputs` for each ship
    // Apply impulses based on the recorded inputs
    for (player, mut transform, mut rb_imps) in &mut player_info {
        let player_uuid = player.id.clone();
        let control_input = match control_inputs_lock.get(&player_uuid) {
            Some(input) => input,
            None => {
                tracing::trace!(player.id = ?player.id, "No control input found");
                continue;
            }
        };
        let thrust = control_input.thrust;
        let rotation = control_input.rotation;

        // Thrust exerts an impulse on the rigid body along the axis the ship is facing (not traveling)
        // get the ship's forward vector by applying the current rotation to the ships initial facing
        // vector
        let heading = transform.rotation * Vec3::Y;
        // Ignore z axis
        let heading_2d = Vec2::new(heading.x, heading.y);

        // Apply an impulse to the rigid body along the ship axis
        rb_imps.impulse = thrust as f32 * heading_2d * player.impulse;

        // Apply a torque impulse to the rigid body
        rb_imps.torque_impulse = rotation as f32 * player.torque_impulse;
    }
}

pub fn update_public_game_state_system(
    app_state: Res<AppState>,
    query: Query<(&ControllableShip, &Transform, &Velocity)>,
) {
    let mut active_game_lock = app_state.active_game.lock().unwrap();
    let active_game = match active_game_lock.as_mut() {
        Some(game) => game,
        None => {
            tracing::debug!("No active game, nothing to update");
            return;
        }
    };

    // Build a new set of ships from the ECS data
    let mut ships: Vec<Ship> = Vec::new();

    for (player, transform, velocity) in query.iter() {
        // Find the player UUID from the game's ship entities.
        let player_uuid = player.id.clone();
        tracing::trace!(player.id = ?player.id, "Getting ship state");

        let pos = transform.translation;
        let vel = velocity.linvel;
        let ang_vel = velocity.angvel;

        let ship = Ship {
            id: player_uuid,
            position: (pos.x, pos.y),
            velocity: (vel.x, vel.y),
            orientation: transform.rotation.to_euler(EulerRot::XYZ).2,
            angular_velocity: ang_vel,
        };
        tracing::trace!(?ship, "Adding ship to game state");
        ships.push(ship);
    }
    active_game.ships = ships;
}
