use bevy::prelude::*;

/// Biological species classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Species {
    EColi,          // Mesophile
    Listeria,       // Psychrotroph
}

/// Immutable DNA Component representing the genomic traits of the bacterium.
/// By storing immutable genetic traits separately from the fast-changing position/velocity,
/// we preserve cache efficiency when iterating over physical/metabolic systems.
#[derive(Component, Debug, Clone)]
pub struct Dna {
    pub species: Species,

    // --- TEMPERATURE (Ratkowsky model parameters) ---
    pub temp_min: f32,
    pub temp_max: f32,
    pub temp_opt: f32,
    /// Constant parameter b for the square root growth model.
    pub b_growth: f32,
    /// Constant parameter c for the high temperature inhibition factor.
    pub c_growth: f32,

    // --- pH LIMITS (Cardinal pH Model - CPMI) ---
    pub ph_min: f32,
    pub ph_max: f32,
    pub ph_opt: f32,

    // --- CHEMOTAXIS & MOTILITY ---
    pub base_speed: f32,
    pub sensory_radius: f32,
    pub base_tumble_rate: f32, // Base probability or frequency of tumbling (Hz)

    // --- REPRODUCTION ---
    pub repro_threshold: f32,  // Energy required to trigger mitosis (division)
    pub base_metabolic_cost: f32, // Base energy consumption rate per tick
}

impl Dna {
    /// Factory for Escherichia coli (Mesophile)
    /// - Optimal Temp ~37°C. Cannot grow at low temperatures.
    /// - Highly motile and metabolically fast, but vulnerable to cold and acid.
    pub fn e_coli() -> Self {
        Self {
            species: Species::EColi,
            temp_min: 7.0,   // Cannot grow under ~7°C
            temp_max: 46.0,
            temp_opt: 37.0,
            b_growth: 0.025, // Fast base replication rate
            c_growth: 0.04,
            ph_min: 4.4,     // Vulnerable to strong acids
            ph_max: 9.0,
            ph_opt: 7.0,
            base_speed: 40.0,
            sensory_radius: 50.0,
            base_tumble_rate: 1.0, // Tumbles approx once per second
            repro_threshold: 100.0,
            base_metabolic_cost: 0.15,
        }
    }

    /// Factory for Listeria monocytogenes (Psychrotroph)
    /// - Psychrotrophic: survives and grows down to 0°C.
    /// - Highly resistant to osmotic pressure and low pH.
    /// - Replicates slower compared to E. coli.
    pub fn listeria() -> Self {
        Self {
            species: Species::Listeria,
            temp_min: 0.0,   // Can grow at refrigeration temperatures
            temp_max: 45.0,
            temp_opt: 30.0,
            b_growth: 0.012, // Slower base growth compared to E. coli
            c_growth: 0.035,
            ph_min: 4.0,     // More acid tolerant
            ph_max: 9.6,
            ph_opt: 7.0,
            base_speed: 25.0,     // Slower motility
            sensory_radius: 35.0,
            base_tumble_rate: 0.8,
            repro_threshold: 80.0, // Requires less energy to divide
            base_metabolic_cost: 0.08, // Very efficient base metabolism
        }
    }
}

/// Mutable metabolic status.
/// Keeps energy/health metrics grouped together for quick cache access.
#[derive(Component, Debug, Clone)]
pub struct Metabolism {
    pub energy: f32,
    pub health: f32,             // Range: 0.0 (Dead) to 1.0 (Fully healthy)
    pub age: f32,                // Time alive in seconds
    pub accumulated_stress: f32,  // Integrated stress history for Weibull death model
}

impl Default for Metabolism {
    fn default() -> Self {
        Self {
            energy: 50.0,
            health: 1.0,
            age: 0.0,
            accumulated_stress: 0.0,
        }
    }
}

/// Represents the physical location in 2D space.
/// Using custom, minimal 2D vector wrapper avoids Bevy's heavier Transform component,
/// significantly improving memory footprint and cache efficiency when updating physics.
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Position(pub Vec2);

/// Represents the velocity in 2D space.
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Velocity(pub Vec2);

/// Represents the motor/sensory apparatus used for chemotaxis.
#[derive(Component, Debug, Clone)]
pub struct Motor {
    pub current_direction: Vec2,  // Direction vector for chemotactic run in 2D
    pub run_timer: f32,           // Time remaining in current run
    pub last_attractant_level: f32, // Attractant concentration at previous frame (to calculate dS/dt)
}

impl Default for Motor {
    fn default() -> Self {
        Self {
            current_direction: Vec2::X,
            run_timer: 0.0,
            last_attractant_level: 0.0,
        }
    }
}
