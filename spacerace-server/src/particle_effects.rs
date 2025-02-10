use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use uuid::Uuid;

use crate::app_state::AppState;
use crate::components::ship::ControllableShip;
use crate::control::ShipInput;
use crate::game_logic::ServerState;

use bevy_rapier2d::na::{DimAdd, DimMul, DimSub};
use bevy_rapier2d::pipeline::CollisionEvent;
use std::collections::HashMap;
use std::time::Duration;

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

/// Sparks effect for collisions with obstacles/walls.
#[derive(Resource, Default)]
pub struct SparkEffectHandle {
    pub effect: Handle<EffectAsset>,
}

/// Fireworks effect for crossing the finish line (victory).
#[derive(Resource, Default)]
pub struct FireworksEffectHandle {
    pub effect: Handle<EffectAsset>,
}

/// A marker for ephemeral spark/firework effects we can despawn after a delay.
#[derive(Component)]
pub struct ParticleEffectLifetime {
    timer: Timer,
}

impl Plugin for ParticleEffectsPlugin {
    fn build(&self, app: &mut App) {
        // Add the Hanabi plugin
        app.add_plugins(HanabiPlugin)
            // Create our custom resources that hold the effect handles
            .init_resource::<ThrusterEffectHandles>()
            .init_resource::<SparkEffectHandle>()
            .init_resource::<FireworksEffectHandle>()
            // When the game starts (Active), load effect assets & attach thrusters
            .add_systems(
                OnEnter(ServerState::Active),
                load_effect_assets.before(attach_thruster_effects_to_ships),
            )
            // Note this is referenced in game_logic plugin to ensure it runs after the ships have spawned
            //.add_systems(OnEnter(ServerState::Active), attach_thruster_effects_to_ships,)
            .add_systems(
                Update,
                (
                    update_thruster_effect_system.run_if(in_state(ServerState::Active)),
                    spawn_collision_effects_system.run_if(in_state(ServerState::Active)),
                    despawn_finished_effects_system.run_if(in_state(ServerState::Active)),
                ),
            );

        //(
        // spawn_collision_effect_system.after("handle_collision_events"),
        //)
    }
}

pub(crate) fn load_effect_assets(
    mut commands: Commands,
    mut thruster_res: ResMut<ThrusterEffectHandles>,
    mut spark_res: ResMut<SparkEffectHandle>,
    mut fireworks_res: ResMut<FireworksEffectHandle>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // ----- Thruster effect -----
    // Spawner: On/Off initially, but would be great to dynamically set the actual rate
    // based on how much thrust is applied (0 to e.g. 100).
    let thruster_spawner = Spawner::rate(100.0.into());

    // an expression to define the initial velocity, color, etc.
    let mut writer = ExprWriter::new();

    // We'll place the effect behind the ship
    let init_position = SetAttributeModifier::new(
        Attribute::POSITION,
        writer.lit(Vec3::new(0., -0.5, 0.)).expr(),
    );

    // Particle velocity

    let base_velocity = writer.lit(Vec3::new(0., 0., 0.));
    // Generate a random float in [0,1) and shift it to [-0.5, 0.5)
    let random_value = writer.rand(ScalarType::Float) - writer.lit(0.5);

    // Multiply by a vector to scale the spread
    let random_offset = random_value * writer.lit(Vec3::new(20.0, 20.0, 0.0));

    // Add the base velocity and random offset to get the final velocity
    let final_velocity = base_velocity + random_offset;
    let init_velocity = SetAttributeModifier::new(Attribute::VELOCITY, final_velocity.expr());

    // Give particles a lifetime
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.9).expr());

    // Age starts at 0
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());

    let mut color_gradient1 = Gradient::new();
    // Start bright white
    color_gradient1.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
    // Transition to orange
    color_gradient1.add_key(0.2, Vec4::new(1.0, 0.5, 0.05, 1.0));
    // Then to red
    color_gradient1.add_key(0.5, Vec4::new(1.0, 0.2, 0.0, 1.0));
    // Darken
    color_gradient1.add_key(0.8, Vec4::new(0.3, 0.0, 0.0, 1.0));
    // Fade away
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let thruster_effect = effects.add(
        EffectAsset::new(8192, thruster_spawner, writer.finish())
            .with_name("ThrusterEffect")
            //.with_simulation_space(SimulationSpace::Local)
            .init(init_position)
            .init(init_velocity)
            .init(init_lifetime)
            .init(init_age)
            .render(ColorOverLifetimeModifier {
                gradient: color_gradient1,
            })
            .render(SetSizeModifier {
                size: Vec3::splat(2.0).into(), // px wide
            }),
    );

    thruster_res.effect = thruster_effect;

    // SPARKS (wall collisions)
    // -------------------------
    // Spawn particles once (immediatly) on each collision
    let spark_spawner = Spawner::once(75.0.into(), true);
    let mut spark_writer = ExprWriter::new();
    let spark_init_vel = SetVelocityCircleModifier {
        center: spark_writer.lit(Vec3::ZERO).expr(),
        axis: spark_writer.lit(Vec3::Z).expr(),
        speed: (spark_writer.lit(50.0) * spark_writer.rand(ScalarType::Float)).expr(), // faster than thruster
    };
    let spark_init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, spark_writer.lit(0.9).expr());
    let spark_init_age = SetAttributeModifier::new(Attribute::AGE, spark_writer.lit(0.).expr());
    // Define a color gradient from bright yellow to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.8, 0.0, 1.0)); // bright yellow
    gradient.add_key(0.8, Vec4::new(1.0, 0.2, 0.0, 1.0)); // red
    gradient.add_key(1.0, Vec4::splat(0.));

    let spark_init_position = SetAttributeModifier::new(
        Attribute::POSITION,
        spark_writer.lit(Vec3::ZERO).expr(),
    );

    //let spark_color = spark_writer.prop("color", "").expr();

    let spark_asset = effects.add(
        EffectAsset::new(32768, spark_spawner, spark_writer.finish())
            .with_name("CollisionSparkEffect")
            .init(spark_init_position)
            .init(spark_init_vel)
            .init(spark_init_lifetime)
            .init(spark_init_age)
            .render(ColorOverLifetimeModifier { gradient })
            .render(SetSizeModifier {
                size: Vec3::splat(2.0).into(),
            }),
    );
    spark_res.effect = spark_asset;

    // -------------------------
    // FIREWORKS (finish line)
    // -------------------------
    let fireworks_spawner = Spawner::once(5000.0.into(), true);
    let mut fireworks_writer = ExprWriter::new();
    let fwk_init_pos = SetAttributeModifier::new(
        Attribute::POSITION,
        fireworks_writer.lit(Vec3::ZERO).expr(),
    );
    let fwk_init_pos_modifier = SetPositionSphereModifier {
        center: fireworks_writer.lit(Vec3::ZERO).expr(),
        radius: fireworks_writer.lit(50.0).expr(),
        dimension: ShapeDimension::Volume,
    };
    let fwk_init_vel = SetVelocitySphereModifier {
        center: fireworks_writer.lit(Vec3::ZERO).expr(),
        speed: fireworks_writer.lit(200.0).expr(), // big explosion
    };
    let fwk_init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, fireworks_writer.lit(15.).expr());
    let fwk_init_age = SetAttributeModifier::new(Attribute::AGE, fireworks_writer.lit(0.).expr());

    // Every frame, add a gravity-like acceleration downward
    let accel = fireworks_writer.lit(Vec3::new(0., -3., 0.)).expr();
    let update_accel = AccelModifier::new(accel);

    // Firework color gradient
    let mut fwk_init_color = Gradient::new();
    //fwk_init_color.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)); // white
    fwk_init_color.add_key(0.0, Vec4::new(1.0, 0.8, 0.0, 1.0)); // yellow
    fwk_init_color.add_key(0.5, Vec4::new(1.0, 0.2, 0.0, 1.0)); // red
    fwk_init_color.add_key(0.8, Vec4::new(0.3, 0.0, 0.0, 1.0)); // dark red
    fwk_init_color.add_key(1.0, Vec4::splat(0.0)); // fade out

    let fireworks_asset = effects.add(
        EffectAsset::new(8024, fireworks_spawner, fireworks_writer.finish())
            .with_name("FinishFireworksEffect")
            .init(fwk_init_pos)
            .init(fwk_init_pos_modifier)
            .init(fwk_init_vel)
            .init(fwk_init_lifetime)
            .init(fwk_init_age)
            .update(update_accel)
            .render(ColorOverLifetimeModifier {
                gradient: fwk_init_color,
            })
            .render(SetSizeModifier {
                size: Vec3::splat(5.0).into(),
            }),
    );
    fireworks_res.effect = fireworks_asset;

    debug!("Loaded particle effect assets.");
}

pub(crate) fn attach_thruster_effects_to_ships(
    mut commands: Commands,
    thruster_res: Res<ThrusterEffectHandles>,
    ship_query: Query<Entity, With<ControllableShip>>,
) {
    // for ship_ent in &ship_query {
    //     tracing::debug!("Attaching thruster effect to ship {:?}", ship_ent);
    //     // Attach a child for the thruster effect
    //     // We'll put the effect slightly behind the ship on the Y axis.
    //     commands.entity(ship_ent).with_children(|parent| {
    //         parent
    //             .spawn(ParticleEffectBundle {
    //                 effect: ParticleEffect::new(thruster_res.effect.clone())
    //                     .with_z_layer_2d(Some(10.0)), // optional, ensure it renders above/below
    //
    //                 transform: Transform {
    //                     translation: Vec3::new(0.0, -3.0, 0.0),
    //                     //rotation: Quat::from_rotation_z(std::f32::consts::PI),
    //                     ..Default::default()
    //                 },
    //                 ..default()
    //             })
    //             .insert(ShipThrusterEffect); // a marker component so we can query them
    //     });
    // }
    // info!("Spawned thrusters for ships");
}

// Old version
// fn update_thruster_effect_system(
//     mut effects_query: Query<(&Parent, &mut EffectInitializers), With<ShipThrusterEffect>>,
//     ships_query: Query<&ControllableShip>,
//     app_state: Res<AppState>,
// ) {
//     // We'll read the ship's thrust from `app_state.control_inputs`
//     let control_inputs = app_state.control_inputs.lock().unwrap();
//
//     for (parent, mut spawner) in effects_query.iter_mut() {
//         let Ok(ship) = ships_query.get(parent.get()) else {
//             continue;
//         };
//
//         let player_uuid = ship.id;
//         let thrust_input = control_inputs
//             .get(&player_uuid)
//             .map(|input| input.thrust)
//             .unwrap_or(0.0);
//
//         // 0 => no effect, > 0 => thruster effect on
//         // maximum of 600 p/s
//         let rate = if thrust_input > 0.0 {
//             thrust_input * 600.0
//         } else {
//             0.0
//         };
//
//         // TODO modify the particle rate based on the thrust
//         spawner.set_active(rate > 0.0);
//     }
// }

fn update_thruster_effect_system(
    mut commands: Commands,
    thruster_res: Res<ThrusterEffectHandles>,
    ships_query: Query<(&ControllableShip, &Transform)>,
    app_state: Res<AppState>,
) {
    // Read control inputs from shared state.
    let control_inputs = app_state.control_inputs.lock().unwrap();

    // Iterate over every ship (and its global transform).
    for (ship, ship_transform) in ships_query.iter() {
        // Forward Thrusters
        let thrust_input = control_inputs
            .get(&ship.id)
            .map(|input| input.thrust)
            .unwrap_or(0.0);

        if thrust_input > 0.0 {
            // Compute the emitter spawn position:
            // We want the emitter to originate from behind the ship.
            // Assuming the ship's local -Y is the exhaust direction,
            // transform that offset by the ship's rotation.
            let offset = ship_transform.rotation * Vec3::new(0.0, -3.0, 0.0);
            let spawn_position = ship_transform.translation + offset;

            // Spawn a new thruster burst in world space.
            commands.spawn((
                ParticleEffectBundle {
                    effect: ParticleEffect::new(thruster_res.effect.clone())
                        .with_z_layer_2d(Some(10.0)),
                    transform: Transform {
                        translation: spawn_position,
                        rotation: ship_transform.rotation,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                // This component marks the emitter as ephemeral.
                ParticleEffectLifetime {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
            ));
        }

        // Rotational Thrusters
    }
}

/// Spawns ephemeral collision or fireworks effects.
/// Runs after Rapier collision events have fired.
fn spawn_collision_effects_system(
    mut commands: Commands,
    //thruster_res: Res<ThrusterEffectHandles>,
    spark_res: Res<SparkEffectHandle>,
    fireworks_res: Res<FireworksEffectHandle>,
    mut collision_events: EventReader<CollisionEvent>,
    finish_query: Query<&Transform, With<crate::components::FinishRegion>>,
    ship_query: Query<&Transform, With<ControllableShip>>,
) {
    for collision in collision_events.read() {
        match collision {
            // We only care about collisions that actually started
            CollisionEvent::Started(e1, e2, _) => {
                let finish_entity = finish_query
                    .get(*e1)
                    .ok()
                    .map(|_| e1)
                    .or_else(|| finish_query.get(*e2).ok().map(|_| e2));
                let ship_ent = if finish_entity.is_some() {
                    // If e1 was finish, e2 is the ship, or vice versa
                    if finish_query.get(*e1).is_ok() {
                        e2
                    } else {
                        e1
                    }
                } else {
                    // Not a finish collision => maybe a wall or obstacle
                    // We need to figure out which one is the ship.
                    // We'll check if e1 is a ship (has transform) else e2
                    if ship_query.get(*e1).is_ok() {
                        e1
                    } else if ship_query.get(*e2).is_ok() {
                        e2
                    } else {
                        // No ship involved => skip
                        continue;
                    }
                };

                if let Ok(ship_transform) = ship_query.get(*ship_ent) {
                    trace!("Collision with ship {:?}", ship_ent);
                    let collision_pos = ship_transform.translation;

                    if finish_entity.is_some() {
                        info!("Collision with finish line at {:?}", collision_pos);
                        // It's a finish collision => fireworks
                        commands.spawn((
                            ParticleEffectBundle {
                                effect: ParticleEffect::new(fireworks_res.effect.clone())
                                    .with_z_layer_2d(Some(100.0)),
                                transform: Transform {
                                    translation: Vec3::new(0., 0., 20.),
                                    // translation: ship_transform.translation,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ParticleEffectLifetime {
                                timer: Timer::from_seconds(5.0, TimerMode::Once),
                            },
                        ));
                    } else {
                        info!("Collision with obstacle at {:?}", collision_pos);
                        // It's a ship vs. obstacle collision => sparks
                        commands.spawn((
                            ParticleEffectBundle {
                                effect: ParticleEffect::new(spark_res.effect.clone())
                                    .with_z_layer_2d(Some(10.0)),
                                visibility: Visibility::Visible,
                                transform: Transform {
                                    translation: ship_transform.translation,
                                    rotation: ship_transform.rotation,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ParticleEffectLifetime {
                                timer: Timer::from_seconds(3.0, TimerMode::Once),
                            },
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Despawns the ephemeral collision/fireworks particle effect entities
/// once their timer is up.
fn despawn_finished_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ParticleEffectLifetime)>,
) {
    for (ent, mut lifetime) in &mut query {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            commands.entity(ent).despawn_recursive();
        }
    }
}
