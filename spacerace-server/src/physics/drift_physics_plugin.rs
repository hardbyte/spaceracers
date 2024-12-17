use crate::app_state::AppState;
use crate::components;
use bevy::app::{App, Plugin, PostUpdate, Startup, Update};
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Commands, EventReader, KeyCode, Query, Res, Transform};
use bevy_rapier2d::dynamics::ExternalImpulse;
use bevy_rapier2d::pipeline::CollisionEvent::Started;
use bevy_rapier2d::pipeline::{CollisionEvent, ContactForceEvent};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use tracing::{debug, info};

// TODO make these based on the map size
const BOUNDS: Vec2 = Vec2::new(900.0, 640.0);

pub fn setup_physics(mut commands: Commands) {}

pub fn display_events_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        // We are particularly interested in a CollisionEvent
        // between a ship and the finish line - that will trigger the game to end
        // TODO how to get the 2 entities involved?
        debug!("Received collision event: {collision_event:?}");
        if let Started(entity1, entity2, s) = collision_event {
            info!(
                ?s,
                "Collision between entities: {entity1:?} and {entity2:?}"
            );
        }
    }

    // for contact_force_event in contact_force_events.read() {
    //     println!("Received contact force event: {contact_force_event:?}");
    // }
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
        app.add_systems(PostUpdate, display_events_system);
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
