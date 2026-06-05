use bevy::prelude::*;
use crate::components::{Dna, Metabolism, Velocity, Position};
use crate::resources::{Environment, FluidGrid};
use crate::math::{ratkowsky_growth, cpmi_growth, monod_consumption};

/// Metabolism system updating energy, health, stress, and division progress for all bacteria in parallel.
pub fn update_metabolism(
    time: Res<Time>,
    env: Res<Environment>,
    fluid_grid: Res<FluidGrid>,
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
        // 1. Sample local environmental cell from FluidGrid
        let local_cell = fluid_grid.sample_at(b_pos.0);

        // 2. Growth rate scaling based on local Temp (Ratkowsky) and pH (CPMI)
        let t_factor = ratkowsky_growth(
            local_cell.temp,
            b_dna.temp_min,
            b_dna.temp_max,
            b_dna.b_growth,
            b_dna.c_growth,
        );
        let ph_factor = cpmi_growth(
            local_cell.ph,
            b_dna.ph_min,
            b_dna.ph_opt,
            b_dna.ph_max,
        );
        let growth_multiplier = t_factor * ph_factor;

        // 3. Monod Kinetics energy absorption using local nutrient concentration
        let energy_absorbed = monod_consumption(local_cell.food, v_max, k_s) * growth_multiplier * dt;
        b_met.energy += energy_absorbed;

        // 4. Energy decay: base metabolic cost + speed penalty (volumetric metabolic expense)
        let speed_ratio = b_vel.0.length() / b_dna.base_speed;
        let speed_cost = speed_ratio * speed_ratio * 0.05;
        let energy_lost = (b_dna.base_metabolic_cost + speed_cost) * dt;
        
        b_met.energy = (b_met.energy - energy_lost).max(0.0);
        b_met.age += dt;

        // 5. Health & stress calculations using local variables
        if b_met.energy <= 0.0 {
            b_met.health = (b_met.health - 0.2 * dt).max(0.0);
        } else if b_met.health < 1.0 {
            b_met.health = (b_met.health + 0.1 * dt).min(1.0);
        }

        let mut stress_increment = 0.0;
        
        if local_cell.temp > b_dna.temp_opt {
            stress_increment += (local_cell.temp - b_dna.temp_opt) * 0.5;
        } else if local_cell.temp < b_dna.temp_min {
            stress_increment += (b_dna.temp_min - local_cell.temp) * 0.2;
        }

        if local_cell.ph < b_dna.ph_min + 0.5 {
            stress_increment += (b_dna.ph_min + 0.5 - local_cell.ph) * 1.5;
        } else if local_cell.ph > b_dna.ph_max - 0.5 {
            stress_increment += (local_cell.ph - (b_dna.ph_max - 0.5)) * 1.5;
        }

        if local_cell.toxin > 0.0 {
            stress_increment += local_cell.toxin * 5.0;
        }

        if stress_increment > 0.0 {
            b_met.accumulated_stress += stress_increment * dt;
        } else {
            b_met.accumulated_stress = (b_met.accumulated_stress - 0.5 * dt).max(0.0);
        }

        // 6. Mitosis Morphing accumulation
        // Once mitosis starts, division progress increments towards 1.0 (reproduction triggers mitosis)
        let mitosis_duration = 2.0; // takes 2 seconds to divide
        if b_met.division_progress > 0.0 {
            b_met.division_progress += dt / mitosis_duration;
            if b_met.division_progress > 1.0 {
                b_met.division_progress = 1.0;
            }
        } else if b_met.energy >= b_dna.repro_threshold {
            // Initiate the visual and physical mitosis sequence
            b_met.division_progress = 0.01;
        }
    });
}
