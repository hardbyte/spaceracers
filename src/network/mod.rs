use bevy::prelude::Res;
use bevy_tokio_tasks::TokioTasksRuntime;
use tracing::info;
use std::net::SocketAddr;
use bevy::prelude::*;
use crate::app_state::AppState;
use crate::network;

mod api;
mod lobby_route;
mod game_state_route;
mod ship_control_route;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_http);
    }
}

pub fn setup_http(runtime: Res<TokioTasksRuntime>, app_state: Res<AppState>) {
    info!("Setting up HTTP routes");
    let app_state = app_state.clone();
    runtime.spawn_background_task(|_| async move {
        let router = api::create_app(app_state);

        // Run our app with hyper on localhost:5000
        let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        info!("Webserver starting. Listening on {}", addr);
        axum::serve(listener, router).await.unwrap();
    });
}