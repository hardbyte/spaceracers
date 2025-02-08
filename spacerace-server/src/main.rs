mod app_state;
mod game_state;

mod components;
mod game_logic;
mod map;
mod network;
mod physics;
mod telemetry;
mod tests;

mod control;
#[cfg(feature = "ui")]
mod graphics_plugin;
mod lobby_graphics_plugin;
mod particle_effects;

use app_state::AppState;

use bevy::prelude::*;

use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::CollisionEventFlags;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use opentelemetry::global;
use rand::prelude::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::info;
use uuid::Uuid;

#[cfg(feature = "ui")]
use graphics_plugin::GraphicsPlugin;

use game_logic::ServerState;

fn main() {
    telemetry::init();

    let app_state = AppState::new();
    info!("Starting Bevy application");

    // Bevy application - at least during development needs to run in the main thread
    // because it opens a window and runs an EventLoop.
    let mut app = App::new();

    app.insert_resource(app_state)
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(physics::DriftPhysicsPlugin)
        .add_plugins(network::NetworkPlugin)
        .add_plugins(control::ControlPlugin);

    #[cfg(feature = "ui")]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //resolution: bevy::window::WindowResolution::new(1000., 1000.),
                title: "SpaceRacers".to_string(),
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

    app.add_plugins(game_logic::GameLogicPlugin).run();

    info!("Shutting down...");

    global::shutdown_tracer_provider();
}
