struct VertexInput {
    @location(0) position: vec3<f32>,      // Local quad coordinates [-1.0, -1.0] to [1.0, 1.0]
    @location(1) normal: vec3<f32>,        // normal.x contains the instance index as f32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) @interpolate(flat) species: u32,
    @location(2) stress: f32,
    @location(3) energy: f32,
    @location(4) age: f32,
    @location(5) division_progress: f32,
};

struct Uniforms {
    time: f32,
    temp: f32,
    ph: f32,
    toxins: f32,
    active_count: u32,
    padding1: u32,
    padding2: u32,
    padding3: u32,
};

struct BacteriaInstance {
    position: vec2<f32>,
    velocity: vec2<f32>,
    species: u32,       // 0 for E. coli, 1 for Listeria
    stress: f32,
    energy: f32,
    age: f32,
    division_progress: f32,
    padding: f32,
};

struct View {
    view_proj: mat4x4<f32>,
    world_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

@group(2) @binding(0) var<storage, read> instances: array<BacteriaInstance>;
@group(2) @binding(1) var<uniform> uniforms: Uniforms;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let instance_index = u32(input.normal.x);

    // Collapse vertices of inactive instances to cull them
    if (instance_index >= uniforms.active_count) {
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return out;
    }

    // 1. Fetch instance data
    let instance = instances[instance_index];
    out.species = instance.species;
    out.stress = instance.stress;
    out.energy = instance.energy;
    out.age = instance.age;
    out.division_progress = instance.division_progress;
    out.local_pos = input.position.xy;

    // 2. Set scale based on species and metabolic energy
    // Bacteria shrink slightly when low on energy
    let energy_scale = 0.7 + 0.3 * (instance.energy / 100.0);
    var base_scale = vec2<f32>(15.0, 8.0); // E. coli size
    if (instance.species == 1u) {
        base_scale = vec2<f32>(11.0, 8.5); // Listeria size (shorter, thicker)
    }
    var scale = base_scale * energy_scale;
    // Stretch the X-dimension during mitosis to fit the two separating entities
    scale.x = scale.x * (1.0 + instance.division_progress);

    // 3. Compute rotation aligning with velocity vector
    let vel_len = length(instance.velocity);
    var dir = vec2<f32>(1.0, 0.0);
    if (vel_len > 0.1) {
        dir = instance.velocity / vel_len;
    }
    
    // Rotation matrix elements aligning X-axis to velocity direction
    let cos_theta = dir.x;
    let sin_theta = dir.y;
    
    let local_pos_scaled = input.position.xy * scale;
    let rotated_pos = vec2<f32>(
        local_pos_scaled.x * cos_theta - local_pos_scaled.y * sin_theta,
        local_pos_scaled.x * sin_theta + local_pos_scaled.y * cos_theta
    );

    // 4. Translate to world coordinates
    let world_pos = vec3<f32>(instance.position + rotated_pos, 0.0);
    
    // Project to screen space
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

// Procedural high-frequency noise to simulate microscopic flagella and fluid distortion
fn fluid_noise(p: vec2<f32>, time: f32) -> f32 {
    let val = sin(p.x * 8.0 + time * 5.0) * cos(p.y * 8.0 - time * 4.0) +
              sin(p.x * 20.0 - time * 10.0) * cos(p.y * 16.0 + time * 8.0) * 0.4;
    return val * 0.05;
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    if (k <= 0.0) {
        return min(a, b);
    }
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time;

    // A. Biological dimensions
    var half_len = 0.5;
    var radius = 0.28;
    if (input.species == 1u) { // Listeria (shorter, fatter)
        half_len = 0.3;
        radius = 0.35;
    }

    // B. Organic Jitter: apply noise distortion to coordinate space
    let noise = fluid_noise(input.local_pos, time);
    let div_scale = 1.0 + input.division_progress;
    let p = input.local_pos * vec2<f32>(div_scale, 1.0) + vec2<f32>(noise, noise * 0.8);

    // C. Mitosis Morphing (Smooth Minimum Metaball capsule separation)
    // The separation between the two daughter cells increases with division progress
    let separation = input.division_progress * (half_len + radius * 0.5);
    
    // Blending factor decreases to 0.0 as division progress reaches 1.0 (complete separation)
    let k = 0.15 * (1.0 - input.division_progress);

    // Daughter cell 1 (left)
    let cap1_x = clamp(p.x, -half_len - separation, -separation);
    let d1 = length(p - vec2<f32>(cap1_x, 0.0)) - radius;

    // Daughter cell 2 (right)
    let cap2_x = clamp(p.x, separation, half_len + separation);
    let d2 = length(p - vec2<f32>(cap2_x, 0.0)) - radius;

    // Smooth minimum combines the two shapes to simulate pinching and division
    var dist = smin(d1, d2, k);

    // D. Cellular Death: Dissolve the shape when stress is extremely high (> 0.8)
    if (input.stress > 0.8) {
        let dissolve_factor = (input.stress - 0.8) / 0.2;
        let dissolve_noise = sin(p.x * 60.0 + time * 2.0) * cos(p.y * 60.0 - time * 3.0);
        // Perturb SDF boundary to erode the cell wall structure
        dist += abs(dissolve_noise) * 0.18 * dissolve_factor;
    }

    // E. Clip outside pixels (with soft anti-aliased edge)
    let edge_fade = 0.015;
    let alpha = 1.0 - smoothstep(-edge_fade, 0.0, dist);
    if (alpha <= 0.0) {
        discard;
    }

    // F. Rim Lighting & Translucency
    // Inward distance ratio (0.0 at membrane, 1.0 at core center)
    let inward_ratio = clamp(-dist / radius, 0.0, 1.0);
    
    // Fresnel Rim Glow: intensifies near the membrane boundary (dist -> 0)
    let rim_light = pow(1.0 - inward_ratio, 2.2) * 1.5;
    
    // Core Translucency: organic fluid sac gradient
    let core_glow = pow(inward_ratio, 1.5) * 0.6;

    // G. Color Interpolation based on stress
    // E. coli default = Cyan, Listeria default = Neon Green
    var healthy_color = vec3<f32>(0.0, 0.8, 1.0); // Cyan
    if (input.species == 1u) {
        healthy_color = vec3<f32>(0.1, 1.0, 0.3); // Neon Green
    }

    let warning_color = vec3<f32>(1.0, 0.65, 0.0); // Amber/Gold
    let death_color = vec3<f32>(1.0, 0.05, 0.05);  // Crimson Red

    var base_color: vec3<f32>;
    if (input.stress < 0.5) {
        let t = input.stress / 0.5;
        base_color = mix(healthy_color, warning_color, t);
    } else {
        let t = (input.stress - 0.5) / 0.5;
        base_color = mix(warning_color, death_color, t);
    }

    // H. Final Shading Assembly
    // Add pulsing glow to healthy bacteria, and erratic flashing to dying ones
    var pulse = 1.0;
    if (input.stress > 0.8) {
        // High-frequency distress blink
        pulse = 0.6 + 0.4 * sin(time * 35.0);
    } else {
        // Slow biological respiration pulse
        pulse = 0.95 + 0.05 * sin(time * 3.0 + input.age);
    }

    let final_rgb = base_color * (rim_light * 1.3 + core_glow) * pulse;
    
    // If the cell is dissolving, fade it out
    var final_alpha = alpha;
    if (input.stress > 0.9) {
        final_alpha *= (1.0 - input.stress) / 0.1;
    }

    return vec4<f32>(final_rgb, final_alpha);
}
