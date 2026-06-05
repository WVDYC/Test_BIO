use bevy::prelude::*;
use rand::Rng;
use crate::components::{Position, Velocity, Dna, Motor};
use crate::spatial_grid::SpatialGrid;
use crate::resources::Environment;
use crate::systems::spawn::Food;
use crate::math::biased_tumble_probability;

/// Chemotaxis system implementing "Run and Tumble" biased random walk.
pub fn update_chemotaxis(
    time: Res<Time>,
    env: Res<Environment>,
    grid: Res<SpatialGrid>,
    food_query: Query<&Position, With<Food>>,
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
        let mut current_attractant = 0.0;
        let mut nearest_food_vector = Vec2::ZERO;
        let mut min_dist_sq = f32::MAX;

        grid.query_nearby(pos.0, dna.sensory_radius, |ent| {
            if let Ok(f_pos) = food_query.get(ent) {
                let to_food = f_pos.0 - pos.0;
                let dist_sq = to_food.length_squared();
                
                if dist_sq <= dna.sensory_radius * dna.sensory_radius {
                    // Accumulate local concentration score (decreases with distance)
                    current_attractant += (dna.sensory_radius - dist_sq.sqrt()).max(0.0);
                    
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                        nearest_food_vector = to_food;
                    }
                }
            }
        });

        // 2. Compute signal gradient dS/dt
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

            let new_dir = if min_dist_sq < f32::MAX && nearest_food_vector != Vec2::ZERO {
                // Chemotactic bias towards the nearest detected food particle
                let food_dir = nearest_food_vector.normalize_or_zero();
                
                // E. coli (highly motile) has a stronger chemotactic bias (0.6) than Listeria (0.3)
                let bias_factor = match dna.species {
                    crate::components::Species::EColi => 0.6,
                    crate::components::Species::Listeria => 0.3,
                };
                
                ((1.0 - bias_factor) * random_dir + bias_factor * food_dir).normalize_or_zero()
            } else {
                // No food detected: completely random tumbling
                random_dir
            };

            motor.current_direction = new_dir;
            motor.run_timer = rng.gen_range(0.5..2.5); // Reset run duration
            vel.0 = new_dir * dna.base_speed;
        } else {
            // Running: maintain current direction
            vel.0 = motor.current_direction * dna.base_speed;
        }
    }
}
