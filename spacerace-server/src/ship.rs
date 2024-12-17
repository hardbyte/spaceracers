use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ship {
    pub id: Uuid,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub orientation: f32,
    pub angular_velocity: f32,
}
