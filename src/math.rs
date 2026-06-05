/// Mathematical implementations of predictive microbiology equations.
/// Used directly by systems to compute growth, consumption, stress, and motility dynamics.

/// 1. Extended Ratkowsky Square-Root Model for Temperature-Dependent Growth.
/// Returns a multiplier in [0.0, inf) representing temperature growth scaling.
#[inline]
pub fn ratkowsky_growth(temp: f32, temp_min: f32, temp_max: f32, b: f32, c: f32) -> f32 {
    if temp <= temp_min || temp >= temp_max {
        return 0.0;
    }
    
    // sqrt(mu) = b * (T - T_min) * (1 - exp(c * (T - T_max)))
    let inner = b * (temp - temp_min) * (1.0 - (c * (temp - temp_max)).exp());
    if inner <= 0.0 {
        0.0
    } else {
        inner * inner // Return actual growth rate multiplier
    }
}

/// 2. Cardinal pH Model (CPMI) for pH-Dependent Growth.
/// Returns a multiplier in [0.0, 1.0] representing pH compatibility.
#[inline]
pub fn cpmi_growth(ph: f32, ph_min: f32, ph_opt: f32, ph_max: f32) -> f32 {
    if ph <= ph_min || ph >= ph_max {
        return 0.0;
    }
    
    let numerator = (ph - ph_min) * (ph - ph_max);
    let denominator = numerator - (ph - ph_opt) * (ph - ph_opt);
    
    if denominator.abs() < 1e-5 {
        0.0
    } else {
        (numerator / denominator).clamp(0.0, 1.0)
    }
}

/// 3. Monod Kinetics for Nutrient Consumption.
/// Returns consumption rate based on local nutrient concentration S.
#[inline]
pub fn monod_consumption(s: f32, v_max: f32, k_s: f32) -> f32 {
    if s <= 0.0 {
        return 0.0;
    }
    v_max * (s / (k_s + s))
}

/// 4. Non-Linear Weibull-Mafart Death/Inactivation Model (Hazard Rate).
/// Calculates the probability of death during a time step dt given accumulated stress.
/// Uses a hazard function h(t) = (p / delta) * (t / delta)^(p-1) to simulate:
/// - Shoulder effect (p > 1): bacteria accumulate sublethal damage before dying.
/// - Tail effect (p < 1): high initial mortality, leaving a resistant subpopulation.
#[inline]
pub fn weibull_death_probability(accumulated_stress: f32, delta: f32, p: f32, dt: f32) -> f32 {
    if accumulated_stress <= 0.0 {
        return 0.0;
    }
    
    // Hazard rate h(t)
    let term = accumulated_stress / delta;
    let hazard = (p / delta) * term.powf(p - 1.0);
    
    // Probability of event in dt is approx 1 - exp(-h * dt)
    let prob = 1.0 - (-hazard * dt).exp();
    prob.clamp(0.0, 1.0)
}

/// 5. Quorum Sensing Hill Equation.
/// Returns a value in [0.0, 1.0] representing the level of phenotype activation
/// based on local Autoinducer (AI) concentration.
#[inline]
pub fn hill_activation(ai_concentration: f32, k_ai: f32, hill_exponent: f32) -> f32 {
    if ai_concentration <= 0.0 {
        return 0.0;
    }
    let ai_pow = ai_concentration.powf(hill_exponent);
    let k_pow = k_ai.powf(hill_exponent);
    ai_pow / (k_pow + ai_pow)
}

/// 6. Chemotaxis Biased Random Walk (Tumble Probability).
/// Modulates the tumble probability based on the signal gradient (dS/dt).
/// - If gradient is positive (towards attractant), tumble rate decreases.
/// - If gradient is negative (away or flat), tumble rate returns to baseline.
#[inline]
pub fn biased_tumble_probability(
    base_tumble_rate: f32,
    gradient: f32,
    sensitivity: f32,
    dt: f32,
) -> f32 {
    // P_tumble = P_base * exp(-sensitivity * dS/dt)
    let rate = base_tumble_rate * (-sensitivity * gradient).exp();
    // Convert rate (Hz) to probability over dt
    let prob = 1.0 - (-rate * dt).exp();
    prob.clamp(0.05, 0.95) // Keep limits to avoid locking behavior
}
