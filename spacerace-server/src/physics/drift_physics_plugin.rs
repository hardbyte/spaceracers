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
const BOUNDS: Vec2 = Vec2::new(900.0, 640.0);

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
    mut player_info: Query<(&components::ship::ControllableShip, &mut Transform)>,
) {
    for (player, mut transform) in &mut player_info {
        // TODO fix this
        // bound the ship within invisible level bounds
        let extents = Vec3::from((crate::physics::drift_physics_plugin::BOUNDS / 2.0, 0.0));
        transform.translation = transform.translation.min(extents).max(-extents);
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
