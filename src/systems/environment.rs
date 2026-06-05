use bevy::prelude::*;
use rand::Rng;
use crate::components::{Dna, Position, Species};
use crate::resources::{Environment, SimulationAssets};
use crate::systems::spawn::Food;

/// Tag component for the HUD text overlay.
#[derive(Component)]
pub struct HudText;

/// Startup system to spawn the interactive Glassmorphism text overlay HUD.
pub fn spawn_hud(mut commands: Commands) {
    // Spawns a beautiful, sleek cyberpunk telemetry panel in the top-left corner
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            width: Val::Px(280.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.85)), // Deep cyber blue (translucent)
        BorderColor(Color::srgba(0.0, 0.8, 1.0, 0.25)),        // Neon cyan border
        BorderRadius::all(Val::Px(10.0)),
    )).with_children(|parent| {
        // Lab Header
        parent.spawn((
            Text::new("IN SILICO BIOLAB"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.8, 1.0)), // Neon Cyan
        ));
        
        parent.spawn((
            Text::new("Predictive Ecosystem Telemetry"),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.6, 0.7)),
        ));

        // Horizontal divider line
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.8, 1.0, 0.2)),
        ));

        // Dynamic telemetry stats text
        parent.spawn((
            Text::new("Loading sensors..."),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.95, 1.0)),
            HudText,
        ));
    });
}

/// System to handle keyboard inputs and update the environment.
pub fn handle_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut env: ResMut<Environment>,
) {
    let mut changed = false;

    // Up / Down arrows: change Temperature
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        env.temperature = (env.temperature + 1.0).min(60.0);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        env.temperature = (env.temperature - 1.0).max(-10.0);
        changed = true;
    }

    // Left / Right arrows: change pH
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        env.ph = (env.ph + 0.2).min(14.0);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        env.ph = (env.ph - 0.2).max(0.0);
        changed = true;
    }

    // T key: toggle toxins
    if keyboard.just_pressed(KeyCode::KeyT) {
        if env.base_toxin_level > 0.0 {
            env.base_toxin_level = 0.0;
        } else {
            env.base_toxin_level = 0.5;
        }
        changed = true;
    }

    if changed {
        println!(
            "Environment altered: Temp = {:.1}°C, pH = {:.1}, Toxins = {:.1}",
            env.temperature, env.ph, env.base_toxin_level
        );
    }
}

/// System to keep food levels replenished using Mesh2d circle primitives.
pub fn replenish_food(
    mut commands: Commands,
    food_query: Query<&Position, With<Food>>,
    sim_assets: Res<SimulationAssets>,
) {
    let current_food_count = food_query.iter().count();
    let target_food = 500;
    
    if current_food_count < target_food {
        let to_spawn = target_food - current_food_count;
        let mut rng = rand::thread_rng();
        let half_box = 300.0; // matching 600x600 size

        for _ in 0..to_spawn {
            let pos = Vec2::new(
                rng.gen_range(-half_box..half_box),
                rng.gen_range(-half_box..half_box),
            );

            commands.spawn((
                Position(pos),
                Food,
                Mesh2d(sim_assets.food_mesh.clone()),
                MeshMaterial2d(sim_assets.food_material.clone()),
                Transform::from_translation(pos.extend(0.0)),
            ));
        }
    }
}

/// System to update HUD text with current environmental stats and population counts.
pub fn update_hud(
    env: Res<Environment>,
    bacteria_query: Query<&Dna>,
    food_query: Query<Entity, With<Food>>,
    mut hud_query: Query<&mut Text, With<HudText>>,
) {
    if let Ok(mut text) = hud_query.get_single_mut() {
        let mut ecoli_count = 0;
        let mut listeria_count = 0;

        for dna in bacteria_query.iter() {
            match dna.species {
                Species::EColi => ecoli_count += 1,
                Species::Listeria => listeria_count += 1,
            }
        }

        let food_count = food_query.iter().count();
        let toxin_status = if env.base_toxin_level > 0.0 { "ACTIVE [HAZARD]" } else { "NOMINAL" };

        **text = format!(
            "Temperature  : {:.1}°C\n\
             pH Level     : {:.1}\n\
             Toxin Buffer : {}\n\n\
             Populations:\n\
             ├─ E. coli (Cyan)   : {}\n\
             ├─ Listeria (Green) : {}\n\
             └─ Nutrient Orbs    : {}\n\n\
             [Controls]\n\
             ▲/▼ : Adjust Temp (+/- 1°C)\n\
             ◀/▶ : Adjust pH (+/- 0.2)\n\
             T   : Toggle Bio-Toxins",
            env.temperature,
            env.ph,
            toxin_status,
            ecoli_count,
            listeria_count,
            food_count
        );
    }
}
