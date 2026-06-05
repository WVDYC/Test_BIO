#![allow(dead_code)]

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

use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

/// UI configuration for spawning new bacteria agents and using lab tools.
#[derive(Resource, Debug, Clone)]
pub struct SpawnSettings {
    pub species: crate::components::Species,
    pub count: u32,
    pub click_spawn: bool,
    pub active_tool: LabTool,
    pub acid_strength: f32,
    pub probe_temp: f32,
    pub probe_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabTool {
    SpawnBacteria,
    FoodDropper,
    AcidSyringe,
    HeatProbe,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        Self {
            species: crate::components::Species::EColi,
            count: 100,
            click_spawn: true,
            active_tool: LabTool::SpawnBacteria,
            acid_strength: 2.0,
            probe_temp: 60.0,
            probe_radius: 80.0,
        }
    }
}

/// Localized temperature source that can be placed in the sandbox environment.
#[derive(Resource, Debug, Clone)]
pub struct HeatProbe {
    pub pos: Vec2,
    pub temperature: f32,
    pub radius: f32,
    pub active: bool,
}

impl Default for HeatProbe {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            temperature: 60.0,
            radius: 80.0,
            active: false,
        }
    }
}

/// Aligned data for a single fluid grid cell (16 bytes total size).
#[derive(Copy, Clone, Debug, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct FluidCell {
    pub food: f32,
    pub toxin: f32,
    pub ph: f32,
    pub temp: f32,
}

/// Diffusing background fluid grid mapping food, toxins, pH and temperature.
#[derive(Resource, Debug, Clone)]
pub struct FluidGrid {
    pub cells: Vec<FluidCell>,
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f32,
}

impl FluidGrid {
    /// Creates a new FluidGrid covering cols * rows centered at the origin.
    pub fn new(cols: usize, rows: usize, cell_size: f32, default_ph: f32, default_temp: f32) -> Self {
        let cells = vec![
            FluidCell {
                food: 1.0, // base starting food concentration
                toxin: 0.0,
                ph: default_ph,
                temp: default_temp,
            };
            cols * rows
        ];
        Self {
            cells,
            cols,
            rows,
            cell_size,
        }
    }

    /// Helper to sample localized environmental values using bilinear interpolation.
    pub fn sample_at(&self, pos: Vec2) -> FluidCell {
        // Map position from world space [-300.0, 300.0] to grid coordinates [0, cols-1] x [0, rows-1]
        let half_width = (self.cols as f32 * self.cell_size) / 2.0;
        let half_height = (self.rows as f32 * self.cell_size) / 2.0;
        
        let x = ((pos.x + half_width) / self.cell_size).clamp(0.0, (self.cols - 1) as f32);
        let y = ((pos.y + half_height) / self.cell_size).clamp(0.0, (self.rows - 1) as f32);
        
        let x0 = x.floor() as usize;
        let x1 = (x0 + 1).min(self.cols - 1);
        let y0 = y.floor() as usize;
        let y1 = (y0 + 1).min(self.rows - 1);
        
        let tx = x - x0 as f32;
        let ty = y - y0 as f32;
        
        let c00 = &self.cells[y0 * self.cols + x0];
        let c10 = &self.cells[y0 * self.cols + x1];
        let c01 = &self.cells[y1 * self.cols + x0];
        let c11 = &self.cells[y1 * self.cols + x1];
        
        let interpolate = |v00: f32, v10: f32, v01: f32, v11: f32| {
            let r0 = v00 * (1.0 - tx) + v10 * tx;
            let r1 = v01 * (1.0 - tx) + v11 * tx;
            r0 * (1.0 - ty) + r1 * ty
        };
        
        FluidCell {
            food: interpolate(c00.food, c10.food, c01.food, c11.food),
            toxin: interpolate(c00.toxin, c10.toxin, c01.toxin, c11.toxin),
            ph: interpolate(c00.ph, c10.ph, c01.ph, c11.ph),
            temp: interpolate(c00.temp, c10.temp, c01.temp, c11.temp),
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
