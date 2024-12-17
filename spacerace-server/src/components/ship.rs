use bevy::prelude::Component;

#[derive(Component)]
pub struct ControllableShip {
    pub id: uuid::Uuid,

    pub impulse: f32,
    pub torque_impulse: f32,
}
