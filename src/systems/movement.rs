use bevy::prelude::*;
use crate::components::{Position, Velocity, Dna, Motor};
use crate::resources::{Environment, FluidGrid};
use crate::spatial_grid::SpatialGrid;

/// System to update the physical positions based on viscous kinematics, soft repulsion, and Brownian motion.
/// Also synchronizes the custom Position component to Bevy's built-in Transform component.
pub fn update_positions(
    time: Res<Time>,
    env: Res<Environment>,
    grid: Res<SpatialGrid>,
    fluid_grid: Res<FluidGrid>,
    mut query: Query<(Entity, &mut Position, &mut Velocity, &mut Transform, &Dna, &Motor)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }
    
    let box_limit = 300.0; // Half of our 600x600 box boundary
    let drag = 12.0;       // Viscous fluid drag damping
    let stiffness = 280.0; // Soft repulsion stiffness
    let overlap_dist = 10.0; // Sum of radii where soft-body repulsion is active

    query.par_iter_mut().for_each(|(entity, mut pos, mut vel, mut transform, dna, motor)| {
        let mut repulsion_force = Vec2::ZERO;

        // 1. Soft-Body Crowding Repulsion: pushes overlapping cells gently apart
        grid.query_nearby(pos.0, overlap_dist, |entry| {
            if entry.entity == entity {
                return;
            }
            let to_other = entry.position - pos.0;
            let dist = to_other.length();
            if dist < overlap_dist && dist > 0.001 {
                let push_dir = -to_other / dist;
                let force_mag = stiffness * (overlap_dist - dist);
                repulsion_force += push_dir * force_mag;
            }
        });

        // 2. Viscous Kinematics: target velocity relaxation (instant alignment, zero space inertia)
        let target_vel = motor.current_direction * dna.base_speed;
        let relaxation = 1.0 - (-drag * dt).exp();
        let current_vel = vel.0;
        let mut new_vel = current_vel + (target_vel - current_vel) * relaxation;

        // Repulsion forces directly push the bacterium, subject to drag
        new_vel += repulsion_force * dt;
        vel.0 = new_vel;

        // Integrate position
        pos.0 += vel.0 * dt;

        // 3. Brownian Motion: organic twitching scaled by sqrt(dt) and local temp
        let local_cell = fluid_grid.sample_at(pos.0);
        let temp_kelvin = local_cell.temp + 273.15;
        let brownian_coeff = 45.0; // Strength of organic twitching
        let twitch_scale = brownian_coeff * (temp_kelvin / 310.15).sqrt() * dt.sqrt();

        let mut rng = rand::thread_rng();
        use rand::Rng;
        let twitch = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ) * twitch_scale;
        
        pos.0 += twitch;

        // 4. Boundary collision: bounce off the walls of the 2D box
        if pos.0.x > box_limit {
            pos.0.x = box_limit;
            vel.0.x = -vel.0.x.abs();
        } else if pos.0.x < -box_limit {
            pos.0.x = -box_limit;
            vel.0.x = vel.0.x.abs();
        }

        if pos.0.y > box_limit {
            pos.0.y = box_limit;
            vel.0.y = -vel.0.y.abs();
        } else if pos.0.y < -box_limit {
            pos.0.y = -box_limit;
            vel.0.y = vel.0.y.abs();
        }

        // 5. Synchronize rendering transform translation (z=0.0)
        transform.translation = pos.0.extend(0.0);
    });
}
