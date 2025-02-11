use bevy::prelude::Component;

pub mod player;
pub mod ship;

pub use player::{Player, PlayerRegistration};
pub use ship::ControllableShip;

#[derive(Component)]
pub struct Person;

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct FinishRegion;

#[derive(Component)]
pub struct ActiveGameEntity;
