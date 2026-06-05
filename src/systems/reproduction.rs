use bevy::prelude::*;
use rand::Rng;
use crate::components::{Position, Velocity, Dna, Metabolism, Motor};
use crate::resources::Environment;
use crate::math::weibull_death_probability;

/// System that handles bacterial reproduction (mitosis + mutation) and death (Weibull stress model).
pub fn update_reproduction(
    mut commands: Commands,
    time: Res<Time>,
    env: Res<Environment>,
    mut query: Query<(Entity, &Position, &mut Metabolism, &mut Dna, &Velocity, &Motor)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Weibull parameters
    let delta = 30.0; // scale parameter
    let p = 1.8;      // shape parameter

    for (entity, pos, mut met, mut dna, vel, motor) in query.iter_mut() {
        // --- 1. DEATH SYSTEM ---
        let mut died = false;

        // Immediate death from complete health depletion
        if met.health <= 0.0 {
            died = true;
        } else {
            // Weibull-Mafart inactivation based on accumulated environmental stress
            let death_prob = weibull_death_probability(met.accumulated_stress, delta, p, dt);
            if rng.gen::<f32>() < death_prob {
                died = true;
            }
        }

        if died {
            commands.entity(entity).despawn();
            continue; // Skip reproduction checks if dead
        }

        // --- 2. REPRODUCTION SYSTEM (MITOSIS) ---
        if met.energy >= dna.repro_threshold {
            // Mitosis consumes half of parent's energy
            met.energy /= 2.0;

            // Increment parent's generation
            let parent_generation = dna.generation;
            let next_generation = parent_generation + 1;
            dna.generation = next_generation;

            // Child spawn position offset
            let offset_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let offset = Vec2::new(offset_angle.cos(), offset_angle.sin()) * 5.0;
            let child_pos = pos.0 + offset;

            // Mutate child DNA traits (genomic drift) and inherit incremented generation
            let mut child_dna = dna.clone();
            child_dna.generation = next_generation;
            
            child_dna.base_speed = (dna.base_speed + rng.gen_range(-3.0..3.0)).clamp(5.0, 120.0);
            child_dna.sensory_radius = (dna.sensory_radius + rng.gen_range(-4.0..4.0)).clamp(10.0, 150.0);
            child_dna.base_tumble_rate = (dna.base_tumble_rate + rng.gen_range(-0.1..0.1)).clamp(0.1, 5.0);
            child_dna.repro_threshold = (dna.repro_threshold + rng.gen_range(-5.0..5.0)).clamp(30.0, 300.0);
            child_dna.base_metabolic_cost = (dna.base_metabolic_cost + rng.gen_range(-0.01..0.01)).clamp(0.01, 1.0);

            // Spawn the child entity
            commands.spawn((
                Position(child_pos),
                Velocity(vel.0.normalize_or_zero() * child_dna.base_speed),
                child_dna,
                Metabolism {
                    energy: met.energy,
                    health: 1.0,
                    age: 0.0,
                    accumulated_stress: 0.0,
                },
                Motor {
                    current_direction: motor.current_direction,
                    run_timer: rng.gen_range(0.5..2.0),
                    last_attractant_level: 0.0,
                },
                Transform::from_translation(child_pos.extend(0.0)),
            ));
        }
    }
}
