use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        storage::ShaderStorageBuffer,
    },
    sprite::Material2d,
};
use crate::resources::FluidGrid;

/// Custom material for rendering the diffusing fluid background environment.
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
pub struct BackgroundMaterial {
    #[storage(0, visibility(all), read_only)]
    pub grid: Handle<ShaderStorageBuffer>,
    #[uniform(1, visibility(all))]
    pub time: f32,
}

impl Material2d for BackgroundMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }
}

/// Resource holding the handles to the background rendering resources.
#[derive(Resource, Debug, Clone)]
pub struct BackgroundRenderer {
    pub material_handle: Handle<BackgroundMaterial>,
    pub buffer_handle: Handle<ShaderStorageBuffer>,
}

/// System to update the background material's GPU storage buffer and uniforms.
pub fn update_background_shader(
    time: Res<Time>,
    fluid_grid: Res<FluidGrid>,
    renderer: Res<BackgroundRenderer>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // 1. Copy the FluidGrid cells data directly to the GPU storage buffer
    if let Some(buffer) = buffers.get_mut(&renderer.buffer_handle) {
        buffer.set_data(fluid_grid.cells.clone());
    }

    // 2. Update the elapsed time uniform
    if let Some(material) = materials.get_mut(&renderer.material_handle) {
        material.time = time.elapsed_secs();
    }
}
