use bevy::prelude::*;
use rand::Rng;
use crate::components::{Position, Velocity, Dna, Motor};
use crate::resources::{Environment, FluidGrid};
use crate::math::biased_tumble_probability;

/// Chemotaxis system implementing "Run and Tumble" biased random walk using FluidGrid gradients.
pub fn update_chemotaxis(
    time: Res<Time>,
    env: Res<Environment>,
    fluid_grid: Res<FluidGrid>,
    mut query: Query<(&Position, &mut Velocity, &Dna, &mut Motor)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    let mut rng = rand::thread_rng();
    let chemotactic_sensitivity = 5.0; // Sensitivity factor alpha for gradient detection

    for (pos, mut vel, dna, mut motor) in query.iter_mut() {
        // 1. Calculate current attractant (food) level at position
        let sampled = fluid_grid.sample_at(pos.0);
        let current_attractant = sampled.food;

        // 2. Compute signal gradient dS/dt (temporal change)
        let gradient = if dt > 0.0 {
            (current_attractant - motor.last_attractant_level) / dt
        } else {
            0.0
        };
        motor.last_attractant_level = current_attractant;

        // 3. Compute biased tumble probability
        let tumble_prob = biased_tumble_probability(
            dna.base_tumble_rate,
            gradient,
            chemotactic_sensitivity,
            dt,
        );

        motor.run_timer -= dt;

        // 4. Decide whether to tumble
        let should_tumble = motor.run_timer <= 0.0 || rng.gen::<f32>() < tumble_prob;

        if should_tumble {
            // Tumbling: select a new direction
            let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let random_dir = Vec2::new(random_angle.cos(), random_angle.sin());

            // 5. Compute spatial gradient by sampling 4 points in a cross around the bacterium
            let sensory_dist = dna.sensory_radius * 0.5;
            let food_left = fluid_grid.sample_at(pos.0 + Vec2::new(-sensory_dist, 0.0)).food;
            let food_right = fluid_grid.sample_at(pos.0 + Vec2::new(sensory_dist, 0.0)).food;
            let food_down = fluid_grid.sample_at(pos.0 + Vec2::new(0.0, -sensory_dist)).food;
            let food_up = fluid_grid.sample_at(pos.0 + Vec2::new(0.0, sensory_dist)).food;
            
            let grad_vec = Vec2::new(food_right - food_left, food_up - food_down);
            let grad_len = grad_vec.length();

            let new_dir = if grad_len > 0.001 {
                let food_dir = grad_vec / grad_len;
                // E. coli has a stronger chemotactic bias (0.6) than Listeria (0.3)
                let bias_factor = match dna.species {
                    crate::components::Species::EColi => 0.6,
                    crate::components::Species::Listeria => 0.3,
                };
                ((1.0 - bias_factor) * random_dir + bias_factor * food_dir).normalize_or_zero()
            } else {
                random_dir
            };

            motor.current_direction = new_dir;
            motor.run_timer = rng.gen_range(0.5..2.5); // Reset run duration
            
            // Under Low-Reynolds viscous kinematics, velocity is set in movement.rs
            // based on motor.current_direction, so we don't overwrite it here.
            vel.0 = new_dir * dna.base_speed;
        } else {
            vel.0 = motor.current_direction * dna.base_speed;
        }
    }
}
