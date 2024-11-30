mod app;
mod app_state;
mod game_state;
mod player;
mod routes;
mod ship;
mod telemetry;

mod components;

mod tests;

use app_state::AppState;
use axum::extract::State;
use axum::Json;
use opentelemetry::global;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use tracing::info;
use uuid::Uuid;
use crate::components::{Person, Name};

const BOUNDS: Vec2 = Vec2::new(900.0, 640.0);

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Default::default(),
        ..default()
    });
}


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


pub fn display_events(
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

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_graphics);
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    //resolution: bevy::window::WindowResolution::new(1000., 1000.),
                    title: "Graphics Rendering Plugin".to_string(),
                    ..default()
                }),
                ..default()
            }),
            RapierDebugRenderPlugin::default()
        ));
    }
}

pub struct DriftPhysicsPlugin;

impl Plugin for DriftPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0),
        ));

        app.insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )));
        app.add_systems(Startup, setup_physics);
        app.add_systems(Update, ship_movement);
        app.add_systems(PostUpdate, display_events);
    }
}

pub fn ship_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_info: Query<(&crate::components::DemoPlayer, &mut Transform, &mut ExternalImpulse)>,
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

pub fn setup_scene(mut commands: Commands) {
    /*
     * Ground
     */
    let ground_size = 500.0;
    let ground_height = 10.0;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 0.0 * -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height),
    ));

    let sprite_size = 10.0;

    // Spawn entity with `DemoPlayer` struct as a component for access in movement query.
    commands.spawn((
        crate::components::DemoPlayer {
            impulse: 5_000.0,
            torque_impulse: 250.0,
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
        ExternalImpulse::default(),
        AdditionalMassProperties::Mass(100.0),
        Restitution::coefficient(0.9),
        Friction::coefficient(0.5),
        Collider::ball(sprite_size / 2.0),
    ));
}


fn main() {
    telemetry::init();

    // Bevy application
    App::new()
        .add_plugins(DriftPhysicsPlugin)
        .add_plugins(GraphicsPlugin)
        .add_systems(Startup, setup_scene)
        .run();

    info!("Shutting down...");

    global::shutdown_tracer_provider();
}
