use crate::app_state::AppState;
use crate::components;
use bevy::app::{App, Plugin, PostUpdate, Startup, Update};
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::prelude::{Commands, EventReader, KeyCode, Query, Res, Transform};
use bevy_rapier2d::dynamics::{ExternalImpulse, RigidBody};
use bevy_rapier2d::pipeline::CollisionEvent::Started;
use bevy_rapier2d::pipeline::{CollisionEvent, ContactForceEvent};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::rapier::prelude::Collider;
use tracing::info;

// TODO make these based on the map size

pub fn setup_physics(mut commands: Commands) {}

pub fn handle_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    finish_query: Query<&components::FinishRegion>,
    ship_query: Query<&components::ship::ControllableShip>,
    mut commands: Commands,
    app_state: Res<AppState>,
    time: Res<Time>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let finish_entity = if finish_query.get(*entity1).is_ok() {
                *entity1
            } else if finish_query.get(*entity2).is_ok() {
                *entity2
            } else {
                continue;
            };

            let player_entity = if finish_entity == *entity1 {
                *entity2
            } else {
                *entity1
            };

            if let Ok(player) = ship_query.get(player_entity) {
                info!("Player {:?} has finished the race!", player.id);

                // Record the finish time
                let mut active_game_lock = app_state.active_game.lock().unwrap();
                if let Some(active_game) = active_game_lock.as_mut() {
                    let current_time = time.elapsed_secs();
                    active_game
                        .finish_times
                        .entry(player.id)
                        .or_insert(current_time);
                }

                // Despawn the ship as it has finished the race
                commands.entity(player_entity).despawn();
            }
        }
    }
}

pub struct DriftPhysicsPlugin;

impl Plugin for DriftPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0));

        app.add_systems(Startup, setup_physics);

        #[cfg(feature = "ui")]
        {
            app.add_systems(Update, apply_keyboard_controls_system);
        }

        app.add_systems(Update, apply_bounds_system);
        app.add_systems(PostUpdate, handle_collision_events);
    }
}

pub fn apply_bounds_system(
    mut player_info: Query<(&components::ship::ControllableShip, &Sprite, &mut Transform)>,
    app_state: Res<AppState>,
) {
    let active_game_guard = app_state.active_game.lock().unwrap();
    if let Some(active_game) = active_game_guard.as_ref() {
        // The map is centered at (0,0), with `map.size` specifying total width & height.
        // So half of that is the "max" in each axis direction.
        let half_map_width = active_game.map.size.x / 2.0;
        let half_map_height = active_game.map.size.y / 2.0;

        for (player, sprite, mut transform) in &mut player_info {
            // Suppose the ship sprite is, for example, 25.0 x 25.0.
            // This ensures the entire sprite is clamped on-screen.
            // If `custom_size` isn’t set, you can fallback to a default radius or just skip it.
            let half_ship_width = match sprite.custom_size {
                Some(size) => size.x / 2.0,
                None => 0.0, // or a fallback radius
            };
            let half_ship_height = match sprite.custom_size {
                Some(size) => size.y / 2.0,
                None => 0.0,
            };

            // Calculate the min and max coordinates
            // so the entire sprite stays inside the map.
            let min_x = -half_map_width + half_ship_width;
            let max_x = half_map_width - half_ship_width;
            let min_y = -half_map_height + half_ship_height;
            let max_y = half_map_height - half_ship_height;

            // Now clamp each axis
            transform.translation.x = transform.translation.x.clamp(min_x, max_x);
            transform.translation.y = transform.translation.y.clamp(min_y, max_y);

            // Z should remain unchanged, so no clamp for transform.translation.z
        }
    }
}

pub fn apply_keyboard_controls_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: Res<AppState>,
    mut player_info: Query<(
        &components::ship::ControllableShip,
        &mut Transform,
        &mut ExternalImpulse,
    )>,
) {
    for (player, mut transform, mut rb_imps) in &mut player_info {
        let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
        let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
        let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

        // Vector "forwards" if up is pressed, "backwards" if down is pressed
        let thrust = (up as i8) - (down as i8);
        let rotation = (left as i8) - (right as i8);

        // Inject the player's input into the physics simulation as if it had come
        // via the network.
        let mut control_inputs_lock = app_state.control_inputs.lock().unwrap();
        let player_uuid = player.id.clone();
        control_inputs_lock.insert(
            player_uuid,
            crate::control::ShipInput {
                thrust: thrust as f32,
                rotation: rotation as f32,
            },
        );
    }
}
