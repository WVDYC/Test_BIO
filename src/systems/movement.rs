use bevy::prelude::*;
use crate::components::{Position, Velocity};

use crate::resources::Environment;

/// System to update the physical positions based on velocities and clamp/bounce off boundaries.
/// Also synchronizes the custom Position component to Bevy's built-in Transform component.
pub fn update_positions(
    time: Res<Time>,
    env: Res<Environment>,
    mut query: Query<(&mut Position, &mut Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    let box_limit = 300.0; // Half of our 600x600 box boundary

    query.par_iter_mut().for_each(|(mut pos, mut vel, mut transform)| {
        // 1. Euler integration step (2D vectors)
        pos.0 += vel.0 * dt;

        // 2. Boundary collision: bounce off the walls of the 2D box
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

        // 3. Synchronize rendering transform translation (extending 2D to 3D with z=0.0)
        transform.translation = pos.0.extend(0.0);
    });
}
