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

/// Plugin to register all instanced rendering components and shaders.
pub struct InstancedRenderPlugin;

impl Plugin for InstancedRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BacteriaMaterial>::default())
            .add_systems(Startup, setup_instanced_render)
            .add_systems(Update, crate::bacteria_material::update_bacteria_shader_buffers);
    }
}

/// Startup system to setup the HDR camera, bloom effects, and instanced mesh resources.
fn setup_instanced_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BacteriaMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // 1. Perspective HDR Camera with Bloom for bioluminescent glow
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true, // HDR is strictly required for Bloom!
            clear_color: ClearColorConfig::Custom(Color::srgb(0.01, 0.02, 0.04)), // Deep oceanic black
            ..default()
        },
        // Premium glow bloom settings
        Bloom {
            intensity: 0.15,
            ..default()
        },
    ));

    // 2. Preallocate the GPU Storage Buffer containing 100,000 empty instances
    let max_instances = 100_000;
    let dummy_instances = vec![
        BacteriaInstance {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            species: 0,
            stress: 0.0,
            energy: 0.0,
            age: 0.0,
        };
        max_instances
    ];

    let buffer = ShaderStorageBuffer::from(dummy_instances);
    let buffer_handle = buffers.add(buffer);

    // 3. Create the custom material with the storage buffer handle
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

    // 6. Spawn the single GPU instanced rendering entity
    // This entity draws all 100,000 bacteria in a single draw call.
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::default(),
    ));

    println!("GPU Instanced Renderer initialized: preallocated 100,000 slots.");
}

/*
================================================================================
MICROSCOPE LENS POST-PROCESSING SUGGESTION
================================================================================
To achieve scanning-electron microscope vignette and chromatic aberration, Bevy
recommends writing a post-processing custom render pass.
Alternatively, a simple, zero-overhead HUD Vignette overlay can be spawned in Bevy UI:

commands.spawn((
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    },
    // Translucent dark gradient vignette texture
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
    // Blending mode or custom post-process shader
));
================================================================================
*/
