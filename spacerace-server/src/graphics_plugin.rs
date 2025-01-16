use bevy::app::{App, Plugin, PluginGroup, Startup};
use bevy::color::Color;
use bevy::prelude::{default, Camera2d, ClearColor, Commands, OrthographicProjection, Projection};

use crate::lobby_graphics_plugin::LobbyGraphicsPlugin;
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

// I'm using this to visualize the physics engine
// It is likely not part of the final game
pub struct GraphicsPlugin;

fn setup_graphics(mut commands: Commands) {
    commands.spawn((
        Camera2d { ..default() },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                width: 1920.0,
                height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        // Background
        app.insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )));

        app.add_systems(Startup, setup_graphics);

        // TODO make a feature flag
        //app.add_plugins(RapierDebugRenderPlugin::default());

        app.add_plugins(LobbyGraphicsPlugin);
    }
}
