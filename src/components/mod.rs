use bevy::prelude::Component;

#[derive(Component)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
pub struct Person;


#[derive(Component)]
pub struct Name(pub String);

// The float value is the player acceleration in 'pixels/second/second'.
#[derive(Component)]
pub struct ControllableShip {
    pub impulse: f32,
    pub torque_impulse: f32,
}
