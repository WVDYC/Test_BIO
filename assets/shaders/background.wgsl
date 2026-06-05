struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct View {
    view_proj: mat4x4<f32>,
    world_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = input.uv;
    out.clip_position = view.view_proj * vec4<f32>(input.position, 1.0);
    return out;
}

struct FluidCell {
    food: f32,
    toxin: f32,
    ph: f32,
    temp: f32,
};

@group(2) @binding(0) var<storage, read> grid: array<FluidCell>;
@group(2) @binding(1) var<uniform> time: f32;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Bilinear interpolation of the 30x30 grid data
    let cols = 30.0;
    let rows = 30.0;
    
    // Convert UV to grid coordinate space [0, 29]
    // Clamping to [0.5, 29.5] simplifies bilinear boundaries
    let x = input.uv.x * cols;
    let y = (1.0 - input.uv.y) * rows; // Flip Y UV coordinate
    
    let x0 = clamp(floor(x - 0.5), 0.0, cols - 1.0);
    let x1 = clamp(x0 + 1.0, 0.0, cols - 1.0);
    let y0 = clamp(floor(y - 0.5), 0.0, rows - 1.0);
    let y1 = clamp(y0 + 1.0, 0.0, rows - 1.0);
    
    let tx = fract(x - 0.5);
    let ty = fract(y - 0.5);
    
    let idx00 = u32(y0 * cols + x0);
    let idx10 = u32(y0 * cols + x1);
    let idx01 = u32(y1 * cols + x0);
    let idx11 = u32(y1 * cols + x1);
    
    // Sample cells
    let c00 = grid[idx00];
    let c10 = grid[idx10];
    let c01 = grid[idx01];
    let c11 = grid[idx11];
    
    // Bilinear interpolation
    let food = mix(
        mix(c00.food, c10.food, tx),
        mix(c01.food, c11.food, tx),
        ty
    );
    
    let toxin = mix(
        mix(c00.toxin, c10.toxin, tx),
        mix(c01.toxin, c11.toxin, tx),
        ty
    );
    
    let ph = mix(
        mix(c00.ph, c10.ph, tx),
        mix(c01.ph, c11.ph, tx),
        ty
    );

    // Compute visual background colors:
    // Food = Glowing Deep Blue / Teal
    // Toxins = Burning Orange / Crimson
    
    // Deep blue background glow for nutrients
    let food_intensity = clamp(food / 8.0, 0.0, 1.0);
    let food_color = vec3<f32>(0.02, 0.12, 0.35) * food_intensity;
    
    // Burning orange glow for toxins
    let toxin_intensity = clamp(toxin * 1.5, 0.0, 1.0);
    let toxin_color = vec3<f32>(0.45, 0.08, 0.0) * toxin_intensity;
    
    // Combine base fluid layers
    var rgb = food_color + toxin_color;
    
    // Add a very subtle microbial cell fluid background noise (microscopic look)
    let noise = sin(input.uv.x * 300.0 + time * 0.2) * cos(input.uv.y * 300.0 - time * 0.15) * 0.015;
    rgb += vec3<f32>(noise);
    
    // Low baseline background vignette (very dark blue/violet void)
    let dist_to_center = length(input.uv - vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.4, 0.75, dist_to_center);
    
    // Dark microscopical vignette color
    let void_color = vec3<f32>(0.005, 0.005, 0.012);
    rgb = mix(void_color, rgb, vignette);
    
    // pH visual highlights: strong acidity/alkalinity shifts color slightly
    // acid ph < 5: shifts towards toxic neon yellow-green
    // alkaline ph > 9: shifts towards deep purple
    if (ph < 5.0) {
        let acid_factor = clamp((5.0 - ph) / 3.0, 0.0, 1.0);
        rgb = mix(rgb, vec3<f32>(0.15, 0.2, 0.0) * acid_factor, 0.35);
    } else if (ph > 9.0) {
        let alk_factor = clamp((ph - 9.0) / 4.0, 0.0, 1.0);
        rgb = mix(rgb, vec3<f32>(0.12, 0.02, 0.25) * alk_factor, 0.3);
    }
    
    return vec4<f32>(rgb, 1.0);
}
