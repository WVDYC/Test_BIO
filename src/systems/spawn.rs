use bevy::prelude::*;
use rand::Rng;
use crate::resources::FluidGrid;

/// System to setup the initial sandbox environment and seed food concentrations.
pub fn setup_simulation(
    mut fluid_grid: ResMut<FluidGrid>,
) {
    let mut rng = rand::thread_rng();

    // Spawn 15 localized nutrient blooms at startup in the FluidGrid
    let cols = fluid_grid.cols;
    let rows = fluid_grid.rows;

    for _ in 0..15 {
        let center_x = rng.gen_range(0..cols) as f32;
        let center_y = rng.gen_range(0..rows) as f32;
        let bloom_radius = rng.gen_range(4.0..8.0);
        let strength = rng.gen_range(8.0..12.0);

        for y in 0..rows {
            for x in 0..cols {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < bloom_radius {
                    let idx = y * cols + x;
                    // Gaussian-like nutrient distribution
                    let factor = (1.0 - dist / bloom_radius).max(0.0);
                    fluid_grid.cells[idx].food += strength * factor * factor;
                }
            }
        }
    }
    
    println!("Fluid sandbox initialized: seeded 15 organic nutrient blooms.");
}
