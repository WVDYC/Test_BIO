use bevy::prelude::*;
use crate::components::{Position, Velocity, Dna, Metabolism, Species, ClickRipple};
use crate::spatial_grid::SpatialGrid;
use crate::resources::{Environment, HeatProbe};
use crate::math::hill_activation;

/// System that draws background borders, velocity trails, Quorum Sensing glowing auras, localized Heat Probe, and Click Ripples.
pub fn draw_render_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos<'_, '_>,
    env: Res<Environment>,
    grid: Res<SpatialGrid>,
    heat_probe: Res<HeatProbe>,
    bacteria_query: Query<(&Position, &Velocity, &Dna, &Metabolism)>,
    mut ripple_query: Query<(Entity, &mut ClickRipple)>,
) {
    let dt = time.delta_secs() * env.time_scale;

    // 1. Draw Simulation Border (600x600 square)
    // Toxin alert: flash red if toxins are active
    let border_color = if env.base_toxin_level > 0.0 {
        // Pulse border between red and orange
        let pulse = (time.elapsed_secs() * 5.0).sin() * 0.5 + 0.5;
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

    // 3. Draw Localized Heat Probe boundary if active
    if heat_probe.active {
        let pulse = 1.0 + 0.05 * (time.elapsed_secs() * 6.0).sin();
        let probe_color = Color::srgba(1.0, 0.15, 0.05, 0.3);
        
        // Heat Probe Core
        gizmos.circle_2d(heat_probe.pos, 4.0, Color::srgb(1.0, 0.3, 0.0));
        // Outer thermal boundary
        gizmos.circle_2d(heat_probe.pos, heat_probe.radius * pulse, probe_color);
        gizmos.circle_2d(heat_probe.pos, heat_probe.radius * 0.95 * pulse, probe_color.with_alpha(0.15));
    }

    // 4. Update and Draw Click Ripples
    for (entity, mut ripple) in ripple_query.iter_mut() {
        if dt > 0.0 {
            ripple.radius += 180.0 * dt;
        }

        if ripple.radius >= ripple.max_radius {
            commands.entity(entity).despawn();
        } else {
            let progress = ripple.radius / ripple.max_radius;
            let alpha = (1.0 - progress).max(0.0) * 0.45;
            let color = ripple.color.with_alpha(alpha);

            gizmos.circle_2d(ripple.pos, ripple.radius, color);
            gizmos.circle_2d(ripple.pos, ripple.radius * 0.9, color.with_alpha(alpha * 0.5));
        }
    }

    // 5. Draw individual bacterium trails and Quorum Sensing auras
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
