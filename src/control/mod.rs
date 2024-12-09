mod control_plugin;
pub use control_plugin::ControlPlugin;


#[derive(Clone, Debug)]
pub struct ShipInput {
    pub thrust: f32,
    pub rotation: f32,
}

impl Default for ShipInput {
    fn default() -> Self {
        Self {
            thrust: 0.0,
            rotation: 0.0,
        }
    }
}
