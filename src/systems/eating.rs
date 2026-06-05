use bevy::prelude::*;
use bevy::utils::HashSet;
use crate::components::{Position, Dna, Metabolism};
use crate::spatial_grid::SpatialGrid;
use crate::systems::spawn::Food;

/// Sequential system that checks for bacteria close enough to food particles and consumes them.
/// Running this sequentially with a HashSet ensures that food particles are despawned
/// exactly once, eliminating double-despawn warnings in the console.
pub fn update_eating(
    mut commands: Commands,
    grid: Res<SpatialGrid>,
    food_query: Query<&Position, With<Food>>,
    mut bacteria_query: Query<(&Position, &Dna, &mut Metabolism)>,
) {
    let eat_distance_sq = 9.0; // 3.0 units radius squared
    let mut eaten_food = HashSet::new();

    for (b_pos, b_dna, mut b_met) in bacteria_query.iter_mut() {
        let mut closest_food: Option<(Entity, f32)> = None;

        // Query spatial grid for nearby food particles
        grid.query_nearby(b_pos.0, b_dna.sensory_radius, |ent| {
            // Check if this food particle has already been eaten by another bacterium in this frame
            if eaten_food.contains(&ent) {
                return;
            }

            if let Ok(f_pos) = food_query.get(ent) {
                let dist_sq = b_pos.0.distance_squared(f_pos.0);
                if dist_sq <= eat_distance_sq {
                    match closest_food {
                        Some((_, d_sq)) => {
                            if dist_sq < d_sq {
                                closest_food = Some((ent, dist_sq));
                            }
                        }
                        None => {
                            closest_food = Some((ent, dist_sq));
                        }
                    }
                }
            }
        });

        // Eat the closest food particle found
        if let Some((f_entity, _)) = closest_food {
            eaten_food.insert(f_entity);
            commands.entity(f_entity).despawn();
            b_met.energy += 30.0; // Add energy from food consumption
        }
    }
}
