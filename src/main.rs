pub mod components;
pub mod spatial_grid;
pub mod resources;
pub mod math;
pub mod systems;
pub mod ui;
pub mod bacteria_material;
pub mod render_setup;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use spatial_grid::{SpatialGrid, rebuild_spatial_grid};
use resources::{Environment, SpawnSettings};
use systems::spawn::setup_simulation;
use systems::movement::update_positions;
use systems::metabolism::update_metabolism;
use systems::chemotaxis::update_chemotaxis;
use systems::reproduction::update_reproduction;
use systems::eating::update_eating;
use systems::environment::{handle_inputs, replenish_food};
use systems::render_effects::draw_render_effects;
use ui::{update_history, update_scientific_ui, SimulationHistory, handle_mouse_spawning};
use render_setup::InstancedRenderPlugin;

fn main() {
    println!("Initializing In Silico 2D Bacterial Simulation Engine (Instanced rendering + Egui Panel)...");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "In Silico Bacterial Simulation".to_string(),
                ..default()
            }),
            ..default()
        }))
        // Register bevy_egui UI plugin
        .add_plugins(EguiPlugin)
        // Register custom GPU Instancing plugin (Mesh allocations, storage buffers, and WGSL pipeline)
        .add_plugins(InstancedRenderPlugin)
        // Global simulation parameters
        .insert_resource(Environment::default())
        // Spawning configuration settings
        .insert_resource(SpawnSettings::default())
        // Telemetry history resource for egui plotter
        .insert_resource(SimulationHistory::new(100))
        // 2D Spatial Grid matching bounds (-300.0 to 300.0, cell size = 15.0)
        .insert_resource(SpatialGrid::new(-300.0, -300.0, 600.0, 600.0, 15.0))
        // Startup system for spawning initial populations
        .add_systems(Startup, setup_simulation)
        // Update stage systems: split into smaller tuples to comply with Bevy's system parameter limits
        .add_systems(Update, (
            handle_inputs,
            update_positions,
            rebuild_spatial_grid,
            update_eating,
        ))
        .add_systems(Update, (
            update_metabolism,
            update_chemotaxis,
            update_reproduction,
            replenish_food,
        ))
        .add_systems(Update, (
            draw_render_effects,
            update_history,
            update_scientific_ui,
            handle_mouse_spawning,
        ))
        .run();
}
