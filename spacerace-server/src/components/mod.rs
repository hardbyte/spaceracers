use bevy::prelude::Component;

pub mod ship;

#[derive(Component)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
pub struct Person;

#[derive(Component)]
pub struct Name(pub String);
