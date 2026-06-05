use bevy::prelude::*;
use rand::Rng;
use crate::resources::{Environment, FluidGrid};

/// System to handle keyboard inputs and update the environment.
pub fn handle_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut env: ResMut<Environment>,
) {
    let mut changed = false;

    // Up / Down arrows: adjust background Temperature
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        env.temperature = (env.temperature + 1.0).min(60.0);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        env.temperature = (env.temperature - 1.0).max(-10.0);
        changed = true;
    }

    // Left / Right arrows: adjust background pH
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        env.ph = (env.ph + 0.2).min(14.0);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        env.ph = (env.ph - 0.2).max(0.0);
        changed = true;
    }

    // T key: toggle base toxins level
    if keyboard.just_pressed(KeyCode::KeyT) {
        if env.base_toxin_level > 0.0 {
            env.base_toxin_level = 0.0;
        } else {
            env.base_toxin_level = 0.5;
        }
        changed = true;
    }

    if changed {
        println!(
            "Environment altered: Temp = {:.1}°C, pH = {:.1}, Toxins = {:.1}",
            env.temperature, env.ph, env.base_toxin_level
        );
    }
}

/// System to keep fluid nutrients levels replenished in the FluidGrid.
pub fn replenish_food(
    time: Res<Time>,
    env: Res<Environment>,
    mut fluid_grid: ResMut<FluidGrid>,
) {
    let dt = time.delta_secs() * env.time_scale;
    if dt <= 0.0 {
        return;
    }

    // Sum total food concentration
    let total_food: f32 = fluid_grid.cells.iter().map(|c| c.food).sum();
    let target_food = 1000.0;

    if total_food < target_food {
        let deficit = target_food - total_food;
        let to_inject = deficit * 0.15 * dt; // Regenerate 15% of the nutrient deficit per second
        
        let mut rng = rand::thread_rng();
        let cols = fluid_grid.cols;
        let rows = fluid_grid.rows;

        // Add small nutrient droplets randomly
        for _ in 0..8 {
            let rx = rng.gen_range(0..cols);
            let ry = rng.gen_range(0..rows);
            let idx = ry * cols + rx;
            fluid_grid.cells[idx].food += to_inject / 8.0;
        }
    }
}
