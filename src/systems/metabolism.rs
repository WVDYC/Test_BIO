use bevy::prelude::*;
use crate::components::{Position, Dna, Metabolism, Velocity};
use crate::spatial_grid::SpatialGrid;
use crate::resources::Environment;
use crate::systems::spawn::Food;
use crate::math::{ratkowsky_growth, cpmi_growth, monod_consumption};

/// Metabolism system updating energy, health, and stress for all bacteria in parallel.
pub fn update_metabolism(
    time: Res<Time>,
    env: Res<Environment>,
    grid: Res<SpatialGrid>,
    food_query: Query<&Position, With<Food>>,
    mut bacteria_query: Query<(Entity, &Position, &Dna, &mut Metabolism, &Velocity)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    // Parameters for Monod kinetics
    let v_max = 25.0; // Max energy absorption rate
    let k_s = 5.0;    // Half-saturation constant (nutrient density)

    // Parallel iterator for maximum CPU utilization
    bacteria_query.par_iter_mut().for_each(|(_b_entity, b_pos, b_dna, mut b_met, b_vel)| {
        let mut local_food_count = 0.0;

        // 1. Query spatial grid for nearby food particles
        grid.query_nearby(b_pos.0, b_dna.sensory_radius, |ent| {
            if let Ok(f_pos) = food_query.get(ent) {
                let dist_sq = b_pos.0.distance_squared(f_pos.0);
                if dist_sq <= b_dna.sensory_radius * b_dna.sensory_radius {
                    local_food_count += 1.0;
                }
            }
        });

        // 2. Growth rate scaling based on Temp (Ratkowsky) and pH (CPMI)
        let t_factor = ratkowsky_growth(
            env.temperature,
            b_dna.temp_min,
            b_dna.temp_max,
            b_dna.b_growth,
            b_dna.c_growth,
        );
        let ph_factor = cpmi_growth(
            env.ph,
            b_dna.ph_min,
            b_dna.ph_opt,
            b_dna.ph_max,
        );
        let growth_multiplier = t_factor * ph_factor;

        // 3. Monod Kinetics energy absorption
        let energy_absorbed = monod_consumption(local_food_count, v_max, k_s) * growth_multiplier * dt;
        b_met.energy += energy_absorbed;

        // 4. Energy decay: base metabolic cost + speed penalty (volumetric metabolic expense)
        let speed_ratio = b_vel.0.length() / b_dna.base_speed;
        let speed_cost = speed_ratio * speed_ratio * 0.05;
        let energy_lost = (b_dna.base_metabolic_cost + speed_cost) * dt;
        
        b_met.energy = (b_met.energy - energy_lost).max(0.0);
        b_met.age += dt;

        // 5. Health & stress calculations
        if b_met.energy <= 0.0 {
            b_met.health = (b_met.health - 0.2 * dt).max(0.0);
        } else if b_met.health < 1.0 {
            b_met.health = (b_met.health + 0.1 * dt).min(1.0);
        }

        let mut stress_increment = 0.0;
        
        if env.temperature > b_dna.temp_opt {
            stress_increment += (env.temperature - b_dna.temp_opt) * 0.5;
        } else if env.temperature < b_dna.temp_min {
            stress_increment += (b_dna.temp_min - env.temperature) * 0.2;
        }

        if env.ph < b_dna.ph_min + 0.5 {
            stress_increment += (b_dna.ph_min + 0.5 - env.ph) * 1.5;
        } else if env.ph > b_dna.ph_max - 0.5 {
            stress_increment += (env.ph - (b_dna.ph_max - 0.5)) * 1.5;
        }

        if env.base_toxin_level > 0.0 {
            stress_increment += env.base_toxin_level * 5.0;
        }

        if stress_increment > 0.0 {
            b_met.accumulated_stress += stress_increment * dt;
        } else {
            b_met.accumulated_stress = (b_met.accumulated_stress - 0.5 * dt).max(0.0);
        }
    });
}
