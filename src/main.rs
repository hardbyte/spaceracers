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
mod control;

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
use bevy_rapier2d::prelude::CollisionEvent::Started;
use bevy_rapier2d::rapier::prelude::CollisionEventFlags;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use tracing::info;
use uuid::Uuid;

#[cfg(feature = "ui")]
use graphics_plugin::GraphicsPlugin;

use crate::game_logic::{GameEvent, ServerState};



pub fn spawn_ships(
    mut commands: Commands,
    app_state: Res<AppState>,
) {
    // Spawn a Ship for each player in the active GameState
    let sprite_size = 25.0;

    let active_game_guard = app_state.active_game.lock().unwrap();
    if let Some(active_game) = active_game_guard.as_ref() {
        for player in &active_game.players {

            commands.spawn((
                components::ship::ControllableShip {
                    id: player.id,
                    impulse: 8_000.0,
                    torque_impulse: 8_000.0,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(sprite_size, sprite_size)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(-200.0, 150.0, 0.0),
                    ..Default::default()
                },
                RigidBody::Dynamic,
                //ExternalForce::default(),
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
    mut rapier_config: ResMut<RapierConfiguration>,
    app_state: Res<AppState>
) {
    if let Some(active_game) = app_state.active_game.lock().unwrap().as_ref() {
        info!(game_id=?active_game.game_id, "Setting up scene for game");

        let maps = crate::map::load_maps();
        // Select the map by name from the active GameState
        let map = maps.get(&active_game.map_name)
            .expect("Failed to load map by name");

        // Set up gravity using the map specific value
        rapier_config.gravity = Vec2::Y * map.gravity;

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
    } else {
        info!("No active game to set up scene for");
        return;
    }


    // Finish line sensor - TODO load from map data
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
        Collider::cuboid(80.0, 30.0),
        Sensor,
    ));

    // A collider that will generate a test contact event if it goes through the finish line
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(-30.0, 260.0, 0.0)),
        RigidBody::Dynamic,
        Collider::cuboid(10.0, 10.0),
        ActiveEvents::COLLISION_EVENTS,
        ContactForceEventThreshold(10.0),
    ));

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
        .add_plugins(physics::DriftPhysicsPlugin)
        .add_plugins(network::NetworkPlugin)
        .add_plugins(control::ControlPlugin)
        ;
        //.add_event::<GameEvent>();

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
        app.add_plugins(StatesPlugin);
    }

    app
        .init_state::<ServerState>()

        .add_systems(Update, game_logic::game_system.run_if(in_state(ServerState::Active)))

        // See example for states:
        // https://github.com/bevyengine/bevy/blob/latest/examples/games/game_menu.rs

        //.add_systems(OnEnter(ServerState::Active), game_logic::setup_game_state)
        .add_systems(OnEnter(ServerState::Active), setup_scene)
        .add_systems(OnEnter(ServerState::Active), spawn_ships)

        .add_systems(OnEnter(ServerState::Inactive), game_logic::setup_game_scheduler)
        .add_systems(Update, game_logic::game_scheduler_system.run_if(in_state(ServerState::Inactive)))
        .run();

    info!("Shutting down...");

    global::shutdown_tracer_provider();
}
