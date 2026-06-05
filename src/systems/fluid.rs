use bevy::prelude::*;
use crate::resources::{Environment, FluidGrid, FluidCell, HeatProbe};

/// Helper to interpolate linearly between two floats.
#[inline(always)]
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

/// System to update the FluidGrid values using finite difference diffusion (Laplacian stencil).
/// Also applies localized inputs from HeatProbe, AcidSyringe, and FoodDropper.
pub fn update_fluid_grid(
    time: Res<Time>,
    env: Res<Environment>,
    heat_probe: Res<HeatProbe>,
    mut fluid_grid: ResMut<FluidGrid>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    let cols = fluid_grid.cols;
    let rows = fluid_grid.rows;

    // 1. Double buffer cells to prevent reading from currently written cell
    let mut next_cells = fluid_grid.cells.clone();

    // Diffusion coefficients
    let d_food = 0.35;
    let d_toxin = 0.45;
    let d_ph = 0.3;
    let d_temp = 0.6;

    // Recovery/Decay rates
    let ph_neutralize_rate = 0.35;
    let cooling_rate = 0.25;
    let toxin_decay = 0.2;

    // 2. Diffuse food, toxins, pH and temperature using a 5-point Laplacian stencil
    for y in 0..rows {
        for x in 0..cols {
            let idx = y * cols + x;
            let cell = &fluid_grid.cells[idx];

            let get_neighbor = |nx: isize, ny: isize| -> &FluidCell {
                let clamp_x = nx.clamp(0, cols as isize - 1) as usize;
                let clamp_y = ny.clamp(0, rows as isize - 1) as usize;
                &fluid_grid.cells[clamp_y * cols + clamp_x]
            };

            let left = get_neighbor(x as isize - 1, y as isize);
            let right = get_neighbor(x as isize + 1, y as isize);
            let down = get_neighbor(x as isize, y as isize - 1);
            let up = get_neighbor(x as isize, y as isize + 1);

            let laplace_food = left.food + right.food + down.food + up.food - 4.0 * cell.food;
            let laplace_toxin = left.toxin + right.toxin + down.toxin + up.toxin - 4.0 * cell.toxin;
            let laplace_ph = left.ph + right.ph + down.ph + up.ph - 4.0 * cell.ph;
            let laplace_temp = left.temp + right.temp + down.temp + up.temp - 4.0 * cell.temp;

            // Apply food diffusion
            let mut new_food = cell.food + d_food * dt * laplace_food;
            new_food = new_food.clamp(0.0, 15.0);

            // Apply toxin diffusion and decay
            let mut new_toxin = cell.toxin + d_toxin * dt * laplace_toxin;
            new_toxin *= (-toxin_decay * dt).exp();
            new_toxin = new_toxin.clamp(0.0, 10.0);

            // Apply pH diffusion and recovery to global pH level
            let mut new_ph = cell.ph + d_ph * dt * laplace_ph;
            new_ph += (env.ph - new_ph) * ph_neutralize_rate * dt;
            new_ph = new_ph.clamp(2.0, 14.0);

            // Apply temperature diffusion and cooling to global temperature
            let mut new_temp = cell.temp + d_temp * dt * laplace_temp;
            new_temp += (env.temperature - new_temp) * cooling_rate * dt;
            new_temp = new_temp.clamp(-10.0, 80.0);

            next_cells[idx] = FluidCell {
                food: new_food,
                toxin: new_toxin,
                ph: new_ph,
                temp: new_temp,
            };
        }
    }

    // 3. Apply Heat Probe localized temperature fields
    if heat_probe.active {
        let cols_f = cols as f32;
        let rows_f = rows as f32;
        let cell_size = fluid_grid.cell_size;
        let half_width = (cols_f * cell_size) / 2.0;
        let half_height = (rows_f * cell_size) / 2.0;

        for y in 0..rows {
            for x in 0..cols {
                let cell_pos = Vec2::new(
                    (x as f32 + 0.5) * cell_size - half_width,
                    (y as f32 + 0.5) * cell_size - half_height,
                );
                let dist = cell_pos.distance(heat_probe.pos);
                if dist < heat_probe.radius {
                    let idx = y * cols + x;
                    // Linear falloff gradient away from core
                    let factor = (1.0 - dist / heat_probe.radius).max(0.0);
                    next_cells[idx].temp = mix(next_cells[idx].temp, heat_probe.temperature, factor);
                }
            }
        }
    }

    fluid_grid.cells = next_cells;
}
