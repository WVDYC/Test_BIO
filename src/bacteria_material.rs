#![allow(dead_code, unused_imports)]

use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        render_asset::RenderAssetUsages,
        storage::ShaderStorageBuffer,
        mesh::{Indices, PrimitiveTopology},
    },
    sprite::{Material2d, Material2dPlugin},
};
use bytemuck::{Pod, Zeroable};
use crate::components::{Dna, Metabolism, Position, Velocity};
use crate::resources::Environment;

/// GPU instance structure representing a single bacterium.
/// Must match the WGSL structure exactly and align to 16 bytes (total 32 bytes).
/// Derives ShaderType to allow Bevy's buffer update systems to manage GPU layout automatically.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct BacteriaInstance {
    pub position: Vec2,
    pub velocity: Vec2,
    pub species: u32,       // 0 for E. coli, 1 for Listeria
    pub stress: f32,
    pub energy: f32,
    pub age: f32,
    pub division_progress: f32,
    pub padding: f32,
}

/// GPU Uniforms structure aligned to 16-byte boundaries (total 32 bytes).
/// Derives ShaderType to be compatible with Bevy's AsBindGroup macro.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct BacteriaUniforms {
    pub time: f32,
    pub temp: f32,
    pub ph: f32,
    pub toxins: f32,
    pub active_count: u32,
    pub padding1: u32,
    pub padding2: u32,
    pub padding3: u32,
}

/// Custom material for rendering bacteria using GPU storage buffers.
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
pub struct BacteriaMaterial {
    #[storage(0, visibility(all), read_only)]
    pub instances: Handle<ShaderStorageBuffer>,
    #[uniform(1, visibility(all))]
    pub uniforms: BacteriaUniforms,
}

impl Material2d for BacteriaMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/bacteria.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/bacteria.wgsl".into()
    }
}

/// Resource holding the handles to the instanced rendering mesh and material.
#[derive(Resource, Debug, Clone)]
pub struct InstancedRenderer {
    pub mesh_handle: Handle<Mesh>,
    pub material_handle: Handle<BacteriaMaterial>,
    pub buffer_handle: Handle<ShaderStorageBuffer>,
}

/// Procedurally creates a single 2D mesh containing 100,000 quads.
/// To remain compatible with Bevy's default render pipeline, we store the
/// instance index inside the X component of the standard ATTRIBUTE_NORMAL vector.
pub fn create_instanced_mesh(max_instances: usize) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let mut positions = Vec::with_capacity(max_instances * 4);
    let mut normals = Vec::with_capacity(max_instances * 4); // Storing instance index here
    let mut indices = Vec::with_capacity(max_instances * 6);

    for i in 0..max_instances {
        // Quad local coordinate boundaries
        positions.push(Vec3::new(-1.0, -1.0, 0.0));
        positions.push(Vec3::new(1.0, -1.0, 0.0));
        positions.push(Vec3::new(1.0, 1.0, 0.0));
        positions.push(Vec3::new(-1.0, 1.0, 0.0));

        // Store the instance index as f32 in the X channel of normal
        let idx_f = i as f32;
        let norm_val = Vec3::new(idx_f, 0.0, 0.0);
        normals.push(norm_val);
        normals.push(norm_val);
        normals.push(norm_val);
        normals.push(norm_val);

        let base = (i * 4) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// System to update the GPU Storage Buffer and Uniforms every frame.
pub fn update_bacteria_shader_buffers(
    time: Res<Time>,
    env: Res<Environment>,
    renderer: Res<InstancedRenderer>,
    mut materials: ResMut<Assets<BacteriaMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    bacteria_query: Query<(&Position, &Velocity, &Dna, &Metabolism)>,
) {
    // 1. Gather and pack instance data
    let mut packed_instances = Vec::with_capacity(100_000);
    
    for (pos, vel, dna, met) in bacteria_query.iter() {
        let species_id = match dna.species {
            crate::components::Species::EColi => 0u32,
            crate::components::Species::Listeria => 1u32,
        };

        packed_instances.push(BacteriaInstance {
            position: pos.0,
            velocity: vel.0,
            species: species_id,
            // Map accumulated stress to [0.0, 1.0] (cap at 30.0 max)
            stress: (met.accumulated_stress.min(30.0) / 30.0),
            energy: met.energy,
            age: met.age,
            division_progress: met.division_progress,
            padding: 0.0,
        });
    }

    let active_count = packed_instances.len();

    // Pad buffer to avoid empty bindings
    if packed_instances.is_empty() {
        packed_instances.push(BacteriaInstance {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            species: 0,
            stress: 0.0,
            energy: 0.0,
            age: 0.0,
            division_progress: 0.0,
            padding: 0.0,
        });
    }

    // 2. Update ShaderStorageBuffer asset directly
    if let Some(buffer) = buffers.get_mut(&renderer.buffer_handle) {
        buffer.set_data(packed_instances);
    }

    // 3. Update Material Uniforms
    if let Some(material) = materials.get_mut(&renderer.material_handle) {
        material.uniforms = BacteriaUniforms {
            time: time.elapsed_secs(),
            temp: env.temperature,
            ph: env.ph,
            toxins: env.base_toxin_level,
            active_count: active_count as u32,
            padding1: 0,
            padding2: 0,
            padding3: 0,
        };
    }
}
