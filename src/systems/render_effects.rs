use bevy::prelude::*;
use crate::components::{Position, Velocity, Dna, Metabolism, Species};
use crate::spatial_grid::SpatialGrid;
use crate::resources::Environment;
use crate::math::hill_activation;

/// System that draws background grids, borders, velocity trails, and Quorum Sensing pulsing glowing auras.
pub fn draw_render_effects(
    mut gizmos: Gizmos,
    env: Res<Environment>,
    grid: Res<SpatialGrid>,
    bacteria_query: Query<(&Position, &Velocity, &Dna, &Metabolism)>,
) {
    // 1. Draw Simulation Border (600x600 square)
    // Toxin alert: flash red if toxins are active
    let border_color = if env.base_toxin_level > 0.0 {
        // Pulse border between red and orange
        let pulse = (env.temperature * 5.0).sin() * 0.5 + 0.5;
        Color::srgba(1.0, 0.2 * pulse, 0.0, 0.7)
    } else {
        Color::srgba(0.0, 0.8, 1.0, 0.4) // Cyan neon
    };

    gizmos.rect_2d(
        Vec2::ZERO,
        Vec2::splat(600.0),
        border_color,
    );

    // 2. Draw Sci-Fi Background Laboratory Grid Lines
    let grid_step = 100.0;
    let grid_color = Color::srgba(0.0, 0.5, 1.0, 0.05); // Very faint cyan blue
    
    for i in -2..=2 {
        let offset = i as f32 * grid_step;
        // Vertical grid line
        gizmos.line_2d(Vec2::new(offset, -300.0), Vec2::new(offset, 300.0), grid_color);
        // Horizontal grid line
        gizmos.line_2d(Vec2::new(-300.0, offset), Vec2::new(300.0, offset), grid_color);
    }

    // 3. Draw individual bacterium trails and Quorum Sensing auras
    for (pos, vel, dna, met) in bacteria_query.iter() {
        // A. Draw velocity vector trail (neon tail)
        let speed = vel.0.length();
        if speed > 5.0 {
            let trail_length = 0.15; // 150ms tail duration
            let tail_start = pos.0;
            let tail_end = pos.0 - vel.0 * trail_length;
            
            // Fade the tail color towards the end
            let alpha_trail = (speed / dna.base_speed).min(1.0) * 0.5;
            let tail_color = match dna.species {
                Species::EColi => Color::srgba(0.0, 0.8, 1.0, alpha_trail),
                Species::Listeria => Color::srgba(0.2, 1.0, 0.4, alpha_trail),
            };
            
            gizmos.line_2d(tail_start, tail_end, tail_color);
        }

        // B. Quorum Sensing Activation (Hill Equation based on local density)
        let mut local_bact_count = 0.0;
        grid.query_nearby(pos.0, dna.sensory_radius, |_| {
            local_bact_count += 1.0;
        });

        // Parameters for Hill equation activation
        let k_ai = 16.0;         // Activation threshold (16 neighbors in range)
        let hill_exponent = 3.0; // Hill coefficient (cooperative binding)
        let activation = hill_activation(local_bact_count, k_ai, hill_exponent);

        // If Quorum Sensing is activated (>40% threshold), draw a glowing, breathing aura
        if activation > 0.4 {
            let time_val = met.age * 6.0;
            // Pulsing size based on time
            let pulse_radius = (dna.sensory_radius * 0.35) * (1.0 + (time_val.sin() * 0.15));
            
            let aura_color = match dna.species {
                Species::EColi => Color::srgba(0.0, 0.8, 1.0, 0.08 * activation),
                Species::Listeria => Color::srgba(0.2, 1.0, 0.4, 0.08 * activation),
            };

            // Draw concentric glowing rings
            gizmos.circle_2d(pos.0, pulse_radius, aura_color);
            gizmos.circle_2d(pos.0, pulse_radius * 0.8, aura_color);
        }
    }
}
