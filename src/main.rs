mod app_state;
mod game_state;
mod player;

mod components;
mod game_logic;
mod map;
mod network;
mod physics;
mod ship;
mod telemetry;
mod tests;

#[cfg(feature = "ui")]
mod graphics_plugin;

use crate::components::{Name, Person};
use app_state::AppState;
use axum::extract::State;
use axum::Json;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use opentelemetry::global;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use bevy::state::app::StatesPlugin;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use tracing::info;
use uuid::Uuid;

#[cfg(feature = "ui")]
use graphics_plugin::GraphicsPlugin;
use crate::game_logic::{GameEvent, ServerState};

// TODO make these based on the map size
const BOUNDS: Vec2 = Vec2::new(900.0, 640.0);

pub fn setup_physics(mut commands: Commands) {
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
        Collider::cuboid(80.0, 30.0),
        Sensor,
    ));

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 260.0, 0.0)),
        RigidBody::Dynamic,
        Collider::cuboid(10.0, 10.0),
        ActiveEvents::COLLISION_EVENTS,
        ContactForceEventThreshold(10.0),
    ));
}

pub fn display_events_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        println!("Received collision event: {collision_event:?}");
    }

    for contact_force_event in contact_force_events.read() {
        println!("Received contact force event: {contact_force_event:?}");
    }
}

pub struct DriftPhysicsPlugin;

impl Plugin for DriftPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0),));

        app.add_systems(Startup, setup_physics);

        #[cfg(feature = "ui")]
        {
            app.add_systems(Update, apply_keyboard_controls_system);
        }

        app.add_systems(PostUpdate, display_events_system);
    }
}

pub fn apply_keyboard_controls_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_info: Query<(
        &crate::components::ControllableShip,
        &mut Transform,
        &mut ExternalImpulse,
    )>,
) {
    for (player, mut transform, mut rb_imps) in &mut player_info {
        let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
        let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);

        // Up/Down exerts an impulse on the rigid body along the axis the ship is facing (not traveling)
        // get the ship's forward vector by applying the current rotation to the ships initial facing
        // vector
        let heading = transform.rotation * Vec3::Y;
        // Ignore z axis
        let heading_2d = Vec2::new(heading.x, heading.y);

        // Vector "forwards" if up is pressed, "backwards" if down is pressed
        let impulse_ahead = (up as i8) - (down as i8);
        // Apply an impulse to the rigid body along the ship axis
        rb_imps.impulse = impulse_ahead as f32 * heading_2d * player.impulse;

        // Apply a torque impulse to the rigid body
        let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
        let rotation = (left as i8) - (right as i8);
        rb_imps.torque_impulse = rotation as f32 * player.torque_impulse;

        // bound the ship within the invisible level bounds
        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        transform.translation = transform.translation.min(extents).max(-extents);
    }
}

// TODO network control system something like this
// async fn apply_network_controls_system(
//     app_state: Res<AppState>,
//     mut query: Query<&mut RigidBody>,
// ) {
//     let lobby = app_state.lobby_players.lock().unwrap();
//     // Iterate over ships and apply controls
// }

pub fn setup_scene(mut commands: Commands) {
    let maps = crate::map::load_maps();
    let map = &maps[0]; // Select the first map for now

    // Set up gravity using the map specific value
    //-map.gravity

    // Ground
    let ground_size = 500.0;
    let ground_height = 10.0;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height),
    ));

    // Obstacles
    for obstacle in &map.obstacles {
        commands.spawn((
            TransformBundle::from(Transform::from_xyz(
                obstacle.position.x,
                obstacle.position.y,
                0.0,
            )),
            Collider::cuboid(obstacle.size.x / 2.0, obstacle.size.y / 2.0),
        ));
    }

    // // Spawn Ship
    // let sprite_size = 25.0;
    //
    // commands.spawn((
    //     crate::components::ControllableShip {
    //         impulse: 10_000.0,
    //         torque_impulse: 800.0,
    //     },
    //     SpriteBundle {
    //         sprite: Sprite {
    //             color: Color::srgb(0.0, 0.0, 0.0),
    //             custom_size: Some(Vec2::new(sprite_size, sprite_size)),
    //             ..Default::default()
    //         },
    //         transform: Transform::from_xyz(-200.0, 150.0, 0.0),
    //         ..Default::default()
    //     },
    //     RigidBody::Dynamic,
    //     //ExternalForce::default(),
    //     Damping {
    //         linear_damping: 0.2,
    //         angular_damping: 0.95,
    //     },
    //     ExternalImpulse::default(),
    //     AdditionalMassProperties::Mass(200.0),
    //     Restitution::coefficient(0.9),
    //     Friction::coefficient(0.5),
    //     Collider::ball(sprite_size / 2.0),
    // ));
}

fn main() {
    telemetry::init();

    let app_state = AppState::new();
    info!("Starting Bevy application");

    // Bevy application - at least during development needs to run in the main thread
    // because it opens a window and runs an EventLoop.
    let mut app = App::new();

    app
        .insert_resource(app_state)
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(DriftPhysicsPlugin)
        .add_systems(Startup, network::setup_http)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, game_logic::game_scheduler_system)
        .add_event::<GameEvent>();

    #[cfg(feature = "ui")]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //resolution: bevy::window::WindowResolution::new(1000., 1000.),
                title: "SpaceRaceRS Graphics Rendering Plugin".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GraphicsPlugin);
    }
    #[cfg(not(feature = "ui"))]
    {
        app.add_plugins(MinimalPlugins);
    }


    // TODO look at Bevy support for States:
    // https://github.com/bevyengine/bevy/blob/latest/examples/games/game_menu.rs
    app
        .init_state::<ServerState>()
        .run();

    info!("Shutting down...");

    global::shutdown_tracer_provider();
}
