#![allow(dead_code, unused_imports)]

use bevy::{
    prelude::*,
    core_pipeline::bloom::Bloom,
    render::{
        render_resource::*,
        storage::ShaderStorageBuffer,
    },
    sprite::Material2dPlugin,
};
use crate::bacteria_material::{
    create_instanced_mesh, BacteriaInstance, BacteriaMaterial, BacteriaUniforms, InstancedRenderer,
};
use crate::background_material::{
    BackgroundMaterial, BackgroundRenderer, update_background_shader,
};
use crate::resources::FluidGrid;

/// Plugin to register all instanced rendering components, background material, and shaders.
pub struct InstancedRenderPlugin;

impl Plugin for InstancedRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BacteriaMaterial>::default())
            .add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .add_systems(Startup, setup_instanced_render)
            .add_systems(Update, (
                crate::bacteria_material::update_bacteria_shader_buffers,
                update_background_shader,
            ));
    }
}

/// Startup system to setup the HDR camera, bloom effects, background mesh/material, and instanced mesh resources.
fn setup_instanced_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BacteriaMaterial>>,
    mut bg_materials: ResMut<Assets<BackgroundMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    fluid_grid: Res<FluidGrid>,
) {
    // 1. Perspective HDR Camera with Bloom for bioluminescent glow
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true, // HDR is strictly required for Bloom!
            clear_color: ClearColorConfig::Custom(Color::srgb(0.01, 0.015, 0.03)), // Deep oceanic black
            ..default()
        },
        Bloom {
            intensity: 0.15,
            ..default()
        },
    ));

    // 2. Preallocate the GPU Storage Buffer containing 100,000 empty bacteria instances
    let max_instances = 100_000;
    let dummy_instances = vec![
        BacteriaInstance {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            species: 0,
            stress: 0.0,
            energy: 0.0,
            age: 0.0,
            division_progress: 0.0,
            padding: 0.0,
        };
        max_instances
    ];

    let buffer = ShaderStorageBuffer::from(dummy_instances);
    let buffer_handle = buffers.add(buffer);

    // 3. Create the custom bacteria material with the storage buffer handle
    let material_handle = materials.add(BacteriaMaterial {
        uniforms: BacteriaUniforms {
            time: 0.0,
            temp: 37.0,
            ph: 7.0,
            toxins: 0.0,
            active_count: 0,
            padding1: 0,
            padding2: 0,
            padding3: 0,
        },
        instances: buffer_handle.clone(),
    });

    // 4. Generate the 100,000-quad instanced mesh
    let mesh = create_instanced_mesh(max_instances);
    let mesh_handle = meshes.add(mesh);

    // 5. Insert the InstancedRenderer as a global resource
    commands.insert_resource(InstancedRenderer {
        mesh_handle: mesh_handle.clone(),
        material_handle: material_handle.clone(),
        buffer_handle: buffer_handle.clone(),
    });

    // 6. Spawn the single GPU instanced rendering entity for bacteria
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(0.0, 0.0, 1.0), // slightly in front of Z = 0.0
    ));

    // 7. Initialize background fluid grid storage buffer
    let bg_buffer = ShaderStorageBuffer::from(fluid_grid.cells.clone());
    let bg_buffer_handle = buffers.add(bg_buffer);

    let bg_material_handle = bg_materials.add(BackgroundMaterial {
        grid: bg_buffer_handle.clone(),
        time: 0.0,
    });

    commands.insert_resource(BackgroundRenderer {
        material_handle: bg_material_handle.clone(),
        buffer_handle: bg_buffer_handle.clone(),
    });

    // 8. Spawn background quad mesh (600x600) behind everything (Z = -10.0)
    let bg_mesh_handle = meshes.add(Rectangle::new(600.0, 600.0));
    commands.spawn((
        Mesh2d(bg_mesh_handle),
        MeshMaterial2d(bg_material_handle),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    println!("GPU Instanced Renderer and Background Volumetric Grid initialized.");
}
