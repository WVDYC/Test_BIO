use bevy::prelude::*;
use rand::Rng;
use crate::components::{Position, Velocity, Dna, Metabolism, Motor};
use crate::resources::SimulationAssets;

/// Tag component for food particles.
#[derive(Component, Debug, Default)]
pub struct Food;

/// System to setup the 2D scene, camera, and initial populations of bacteria and food.
pub fn setup_simulation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    // 1. Spawn 2D Camera
    commands.spawn(Camera2d::default());

    let mut rng = rand::thread_rng();

    // Define simulation boundaries (2D square of size 600x600 around center)
    let box_size = 600.0;
    let half_box = box_size / 2.0;

    // Create 2D meshes for smooth circular representation
    let ecoli_mesh = meshes.add(Circle::new(3.5));
    let listeria_mesh = meshes.add(Circle::new(2.8));
    let food_mesh = meshes.add(Circle::new(1.5));

    // Create color materials with glowing sci-fi colors
    let ecoli_mat = color_materials.add(ColorMaterial::from(Color::srgb(0.0, 0.8, 1.0))); // Cyan
    let listeria_mat = color_materials.add(ColorMaterial::from(Color::srgb(0.2, 1.0, 0.4))); // Green
    let food_mat = color_materials.add(ColorMaterial::from(Color::srgb(1.0, 0.8, 0.0))); // Amber/Yellow

    // Insert as a global resource so other systems can access handles
    commands.insert_resource(SimulationAssets {
        ecoli_mesh: ecoli_mesh.clone(),
        ecoli_material: ecoli_mat.clone(),
        listeria_mesh: listeria_mesh.clone(),
        listeria_material: listeria_mat.clone(),
        food_mesh: food_mesh.clone(),
        food_material: food_mat.clone(),
    });

    // Spawn 500 E. coli (Blue glowing circles)
    for _ in 0..500 {
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
            Mesh2d(ecoli_mesh.clone()),
            MeshMaterial2d(ecoli_mat.clone()),
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    // Spawn 500 Listeria (Green glowing circles)
    for _ in 0..500 {
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
            Mesh2d(listeria_mesh.clone()),
            MeshMaterial2d(listeria_mat.clone()),
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    // Spawn 500 Food Particles (Yellow glowing circles)
    for _ in 0..500 {
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
