use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use uuid::Uuid;

use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::control::ShipInput;
use crate::game_logic::ServerState;

use bevy_rapier2d::pipeline::CollisionEvent;
use std::time::Duration;
use std::collections::HashMap;


/// This plugin sets up both thruster and collision particle effects
/// once the game goes into the Active state.
pub struct ParticleEffectsPlugin;


/// Marker so we can query the thruster effect entity
#[derive(Component)]
struct ShipThrusterEffect;


#[derive(Resource, Default)]
pub struct ThrusterEffectHandles {
    pub effect: Handle<EffectAsset>,
}

/// Single resource for your collision effect
#[derive(Resource, Default)]
pub struct CollisionEffectHandle {
    pub effect: Handle<EffectAsset>,
}

impl Plugin for ParticleEffectsPlugin {
    fn build(&self, app: &mut App) {
        // Add the Hanabi plugin
        app.add_plugins(HanabiPlugin)

        // Create our custom resources that hold the effect handles
        .init_resource::<ThrusterEffectHandles>()
            .init_resource::<CollisionEffectHandle>()

        // When the game starts (Active), load effect assets & attach thrusters
        .add_systems(OnEnter(ServerState::Active), load_effect_assets)

        // Also on enter, after ships have spawned, attach thrusters to them
        // .after(your_ship_spawn_system) if you need guaranteed order
        .add_systems(OnEnter(ServerState::Active), attach_thruster_effects_to_ships)

        // Every frame in Active:
        // - update thruster effect intensity based on input
        // - spawn collision sparks after collisions
        //   (we run it after your existing handle_collision_events system
        //    so we know which collisions actually happened)
        .add_systems(Update, update_thruster_effect_system.run_if(in_state(ServerState::Active)));

            //(

                // spawn_collision_effect_system.after("handle_collision_events"),
            //)


    }
}


fn load_effect_assets(
    mut commands: Commands,
    mut thruster_res: ResMut<ThrusterEffectHandles>,
    mut collision_res: ResMut<CollisionEffectHandle>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // ----- Thruster effect -----
    // Spawner: 0 p/s initially, but we’ll dynamically set the actual rate
    // based on how much thrust is applied (0 to e.g. 100).
    let thruster_spawner = Spawner::rate(0.0.into());

    // Setup an expression to define the initial velocity, color, etc.
    let mut writer = ExprWriter::new();

    // We'll place the effect behind the ship, so we can set a small initial velocity
    // in the negative Y direction, for instance. But we can also apply offset/rotation in a parent transform.
    let init_velocity = SetAttributeModifier::new(
        Attribute::VELOCITY,
        writer.lit(Vec3::new(0., -10., 0.)).expr(),
    );

    // Give particles a lifetime of 0.7 sec
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.7).expr());

    // Age starts at 0
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());

    // Color fade over time (optional). For simplicity, let's just set color at spawn.
    let init_color = SetAttributeModifier::new(
        Attribute::COLOR,
        writer.lit(Vec4::new(1., 0.5, 0., 1.)).expr(), // orange color
    );

    // Basic effect with 8192 max particles, continuous spawner (0 p/s initially),
    // and the modifiers from above
    let thruster_effect = effects.add(
        EffectAsset::new(8192, thruster_spawner, writer.finish())
            .with_name("ThrusterEffect")
            .init(init_velocity)
            .init(init_lifetime)
            .init(init_age)
            .init(init_color)
            // You can add a size or scale over lifetime if desired
            .render(SetSizeModifier {
                size: Vec3::splat(2.0).into(), // 2 px wide
            }),
    );

    thruster_res.effect = thruster_effect;

    // ----- Collision effect -----
    // We'll spawn ~30 particles once, each time a collision occurs
    let collision_spawner = Spawner::once(30.0.into(), false);

    let mut writer2 = ExprWriter::new();
    let init_collision_vel = SetVelocitySphereModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        speed: writer2.lit(2.).expr(),
    };
    let init_collision_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer2.lit(0.5).expr());
    let init_collision_age = SetAttributeModifier::new(Attribute::AGE, writer2.lit(0.).expr());
    let init_collision_color = SetAttributeModifier::new(
        Attribute::COLOR,
        writer2.lit(Vec4::new(1., 1., 0., 1.)).expr(), // bright yellow
    );

    let collision_effect = effects.add(
        EffectAsset::new(1024, collision_spawner, writer2.finish())
            .with_name("CollisionEffect")
            .init(init_collision_vel)
            .init(init_collision_lifetime)
            .init(init_collision_age)
            .init(init_collision_color)
            .render(SetSizeModifier {
                size: Vec3::splat(2.0).into(),
            }),
    );

    collision_res.effect = collision_effect;

    info!("Loaded thruster and collision effect assets.");
}


fn attach_thruster_effects_to_ships(
    mut commands: Commands,
    thruster_res: Res<ThrusterEffectHandles>,
    ship_query: Query<Entity, With<ControllableShip>>,
) {
    for ship_ent in &ship_query {
        // Attach a child for the thruster effect
        // We'll put the effect slightly behind the ship on the Y axis.
        commands.entity(ship_ent).with_children(|parent| {
            parent
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(thruster_res.effect.clone())
                        .with_z_layer_2d(Some(1.0)), // optional, ensure it renders above/below
                    transform: Transform::from_translation(Vec3::new(0.0, -12.0, 0.0)), // behind ship
                    ..default()
                })
                .insert(ShipThrusterEffect); // a marker component so we can query them
        });
    }
    info!("Spawned thruster child effect for each ship.");
}

fn update_thruster_effect_system(
    mut effects_query: Query<(&Parent, &mut ParticleEffect), With<ShipThrusterEffect>>,
    ships_query: Query<&ControllableShip>,
    app_state: Res<AppState>,
) {
    // We'll read the ship's thrust from `app_state.control_inputs`
    let control_inputs = app_state.control_inputs.lock().unwrap();

    for (parent, mut spawner) in effects_query.iter_mut() {
        let Ok(ship) = ships_query.get(parent.get()) else { continue };

        let player_uuid = ship.id;
        let thrust_input = control_inputs
            .get(&player_uuid)
            .map(|input| input.thrust)
            .unwrap_or(0.0);

        // 0 => no effect, > 0 => thruster effect on
        // maximum of 60 p/s
        let rate = if thrust_input > 0.0 { 60.0 } else { 0.0 };

        // TODO AI use the rate

    }
}
