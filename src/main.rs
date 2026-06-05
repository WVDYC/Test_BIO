pub mod components;
pub mod spatial_grid;
pub mod resources;
pub mod math;
pub mod systems;

use bevy::prelude::*;
use spatial_grid::{SpatialGrid, rebuild_spatial_grid};
use resources::Environment;
use systems::spawn::setup_simulation;
use systems::movement::update_positions;
use systems::metabolism::update_metabolism;
use systems::chemotaxis::update_chemotaxis;
use systems::reproduction::update_reproduction;
use systems::eating::update_eating;
use systems::environment::{spawn_hud, handle_inputs, replenish_food, update_hud};
use systems::render_effects::draw_render_effects;

fn main() {
    println!("Initializing In Silico 2D Bacterial Simulation Engine (Full Bio-Engine + Sci-Fi GUI)...");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "In Silico Bacterial Simulation".to_string(),
                ..default()
            }),
            ..default()
        }))
        // Global simulation environmental parameters
        .insert_resource(Environment::default())
        // Initialize 2D Spatial Grid matching bounds (-300.0 to 300.0, cell size = 15.0)
        .insert_resource(SpatialGrid::new(-300.0, -300.0, 600.0, 600.0, 15.0))
        // Startup systems
        .add_systems(Startup, (
            setup_simulation,
            spawn_hud,
        ))
        // Update stage systems
        .add_systems(Update, (
            handle_inputs,
            update_positions,
            rebuild_spatial_grid,
            update_eating,
            update_metabolism,
            update_chemotaxis,
            update_reproduction,
            replenish_food,
            update_hud,
            draw_render_effects,
        ))
        .run();
}
