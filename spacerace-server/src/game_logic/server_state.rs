use bevy::prelude::States;

// Enum that will be used as a global state for the game server
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ServerState {
    #[default]
    Inactive,
    Active,
}
