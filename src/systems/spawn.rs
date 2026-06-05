use bevy::prelude::*;
use rand::Rng;
use crate::components::{Position, Velocity, Dna, Metabolism, Motor};
use crate::resources::SimulationAssets;

/// Tag component for food particles.
#[derive(Component, Debug, Default)]
pub struct Food;

/// System to setup the initial populations of bacteria and food.
pub fn setup_simulation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    // Define simulation boundaries (2D square of size 600x600 around center)
    let box_size = 600.0;
    let half_box = box_size / 2.0;

    // Create 2D mesh and material for food particles
    let food_mesh = meshes.add(Circle::new(1.5));
    let food_mat = color_materials.add(ColorMaterial::from(Color::srgb(1.0, 0.8, 0.0))); // Yellow

    // Insert SimulationAssets so replenish_food system can access food handles
    commands.insert_resource(SimulationAssets {
        food_mesh: food_mesh.clone(),
        food_material: food_mat.clone(),
        ecoli_mesh: Handle::default(), // unused placeholder
        ecoli_material: Handle::default(), // unused placeholder
        listeria_mesh: Handle::default(), // unused placeholder
        listeria_material: Handle::default(), // unused placeholder
    });

    // Spawn 5,000 E. coli (Cyan - data-only ECS entities)
    for _ in 0..5000 {
        let pos = Vec2::new(
            rng.gen_range(-half_box..half_box),
            rng.gen_range(-half_box..half_box),
        );
        let dna = Dna::e_coli();
        let vel_dir = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ).normalize_or_zero();

        commands.spawn((
            Position(pos),
            Velocity(vel_dir * dna.base_speed),
            dna,
            Metabolism::default(),
            Motor {
                current_direction: vel_dir,
                run_timer: rng.gen_range(0.5..2.0),
                last_attractant_level: 0.0,
            },
            // Note: No mesh/material components spawned here!
            // The InstancedRenderer draws them all in a single GPU call.
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    // Spawn 5,000 Listeria (Green - data-only ECS entities)
    for _ in 0..5000 {
        let pos = Vec2::new(
            rng.gen_range(-half_box..half_box),
            rng.gen_range(-half_box..half_box),
        );
        let dna = Dna::listeria();
        let vel_dir = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ).normalize_or_zero();

        commands.spawn((
            Position(pos),
            Velocity(vel_dir * dna.base_speed),
            dna,
            Metabolism::default(),
            Motor {
                current_direction: vel_dir,
                run_timer: rng.gen_range(0.5..2.0),
                last_attractant_level: 0.0,
            },
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    // Spawn 1,000 Food Particles (Yellow circular meshes)
    for _ in 0..1000 {
        let pos = Vec2::new(
            rng.gen_range(-half_box..half_box),
            rng.gen_range(-half_box..half_box),
        );

        commands.spawn((
            Position(pos),
            Food,
            Mesh2d(food_mesh.clone()),
            MeshMaterial2d(food_mat.clone()),
            Transform::from_translation(pos.extend(0.0)),
        ));
    }
}
