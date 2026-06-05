use bevy::prelude::*;

/// Global environment parameters regulating bacterial growth, metabolism, and death.
#[derive(Resource, Debug, Clone)]
pub struct Environment {
    /// Temperature in Celsius (°C). Controls growth rate via Ratkowsky model.
    pub temperature: f32,
    /// pH level (0.0 to 14.0). Controls growth rate via CPMI model.
    pub ph: f32,
    /// Global concentration of toxins (0.0 to 1.0). Controls death rate via Weibull model.
    pub base_toxin_level: f32,
    /// Speed factor for the simulation.
    pub time_scale: f32,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            temperature: 37.0,      // E. coli optimum
            ph: 7.0,               // Neutral pH
            base_toxin_level: 0.0,
            time_scale: 1.0,
        }
    }
}

/// UI configuration for spawning new bacteria agents.
#[derive(Resource, Debug, Clone)]
pub struct SpawnSettings {
    pub species: crate::components::Species,
    pub count: u32,
    pub click_spawn: bool,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        Self {
            species: crate::components::Species::EColi,
            count: 100,
            click_spawn: true,
        }
    }
}

/// Shared biological asset handles to avoid asset recreation in runtime systems.
#[derive(Resource, Clone, Debug)]
pub struct SimulationAssets {
    pub ecoli_mesh: Handle<Mesh>,
    pub ecoli_material: Handle<ColorMaterial>,
    pub listeria_mesh: Handle<Mesh>,
    pub listeria_material: Handle<ColorMaterial>,
    pub food_mesh: Handle<Mesh>,
    pub food_material: Handle<ColorMaterial>,
}
