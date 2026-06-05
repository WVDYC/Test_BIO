use bevy::prelude::*;
use crate::components::{Position, Dna, Metabolism};
use crate::resources::{Environment, FluidGrid};

/// System that checks for bacteria overlapping with food concentration in the FluidGrid and consumes it.
pub fn update_eating(
    time: Res<Time>,
    env: Res<Environment>,
    mut fluid_grid: ResMut<FluidGrid>,
    mut bacteria_query: Query<(&Position, &Dna, &mut Metabolism)>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    for (pos, dna, mut met) in bacteria_query.iter_mut() {
        let half_width = (fluid_grid.cols as f32 * fluid_grid.cell_size) / 2.0;
        let half_height = (fluid_grid.rows as f32 * fluid_grid.cell_size) / 2.0;

        let cell_x = ((pos.0.x + half_width) / fluid_grid.cell_size).clamp(0.0, (fluid_grid.cols - 1) as f32) as usize;
        let cell_y = ((pos.0.y + half_height) / fluid_grid.cell_size).clamp(0.0, (fluid_grid.rows - 1) as f32) as usize;
        let idx = cell_y * fluid_grid.cols + cell_x;

        let cell_food = fluid_grid.cells[idx].food;
        if cell_food > 0.0 {
            // E. coli is faster at consuming nutrients than Listeria
            let base_eat_rate = match dna.species {
                crate::components::Species::EColi => 8.0,
                crate::components::Species::Listeria => 5.0,
            };
            let max_eaten = base_eat_rate * dt;
            let eaten = cell_food.min(max_eaten);
            fluid_grid.cells[idx].food -= eaten;
            met.energy += eaten * 15.0;
        }
    }
}
