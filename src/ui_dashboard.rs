use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::{Dna, Metabolism, Species, Position, Velocity, Motor};
use crate::resources::{Environment, SpawnSettings};
use crate::systems::spawn::Food;
use rand::Rng;

/// Resource storing historical data for telemetry plots.
#[derive(Resource, Debug, Default)]
pub struct SimulationHistory {
    pub ecoli_counts: Vec<f32>,
    pub listeria_counts: Vec<f32>,
    pub max_history: usize,
}

impl SimulationHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            ecoli_counts: Vec::new(),
            listeria_counts: Vec::new(),
            max_history,
        }
    }

    pub fn push(&mut self, ecoli: f32, listeria: f32) {
        self.ecoli_counts.push(ecoli);
        self.listeria_counts.push(listeria);
        
        if self.ecoli_counts.len() > self.max_history {
            self.ecoli_counts.remove(0);
            self.listeria_counts.remove(0);
        }
    }
}

/// System to update the simulation history buffer.
pub fn update_history(
    mut history: ResMut<SimulationHistory>,
    bacteria_query: Query<&Dna>,
) {
    let mut ecoli = 0.0;
    let mut listeria = 0.0;
    for dna in bacteria_query.iter() {
        match dna.species {
            Species::EColi => ecoli += 1.0,
            Species::Listeria => listeria += 1.0,
        }
    }
    history.push(ecoli, listeria);
}

/// Systems to render the egui side panel scientific control dashboard.
pub fn update_scientific_ui(
    mut contexts: EguiContexts,
    mut env: ResMut<Environment>,
    mut spawn_settings: ResMut<SpawnSettings>,
    mut commands: Commands,
    history: Res<SimulationHistory>,
    bacteria_query: Query<(&Dna, &Metabolism)>,
    food_query: Query<Entity, With<Food>>,
) {
    let ctx = contexts.ctx_mut();
    
    // Set a sleek dark cyber theme for Egui
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgba_unmultiplied(10, 15, 25, 230); // Translucent deep navy
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 200, 255); // Cyan active
    ctx.set_visuals(visuals);

    egui::SidePanel::left("sci_telemetry_panel")
        .default_width(280.0)
        .frame(egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(5, 8, 15, 220))
            .inner_margin(12.0)
            .outer_margin(10.0)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 200, 255, 60))))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("IN SILICO BIOLAB")
                    .color(egui::Color32::from_rgb(0, 200, 255))
                    .size(20.0)
                    .strong());
                ui.label(egui::RichText::new("Ecosystem Control Panel")
                    .color(egui::Color32::from_rgb(130, 150, 170))
                    .size(11.0));
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // --- SECTION 1: ENVIRONMENTAL CONTROL ---
            ui.label(egui::RichText::new("ENVIRONMENT REGULATION").strong().color(egui::Color32::from_rgb(0, 200, 255)));
            ui.add_space(5.0);
            
            // Temperature slider
            ui.horizontal(|ui| {
                ui.label("Temp:");
                ui.add(egui::Slider::new(&mut env.temperature, 0.0..=50.0)
                    .suffix(" °C")
                    .show_value(true));
            });
            
            // pH Slider
            ui.horizontal(|ui| {
                ui.label("pH:    ");
                ui.add(egui::Slider::new(&mut env.ph, 2.0..=14.0)
                    .show_value(true));
            });

            // Speed (Time Scale) Slider
            ui.horizontal(|ui| {
                ui.label("Speed: ");
                ui.add(egui::Slider::new(&mut env.time_scale, 0.0..=100.0)
                    .suffix("x")
                    .show_value(true));
            });

            // Toxin Toggle
            ui.horizontal(|ui| {
                ui.label("Toxins:");
                let mut toxin_active = env.base_toxin_level > 0.0;
                if ui.checkbox(&mut toxin_active, "Activate Bio-Toxins").changed() {
                    env.base_toxin_level = if toxin_active { 0.5 } else { 0.0 };
                }
            });
            
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // --- SECTION 1.5: SPAWNING CONTROL ---
            ui.label(egui::RichText::new("AGENT SPAWNING").strong().color(egui::Color32::from_rgb(0, 200, 255)));
            ui.add_space(5.0);

            // Select Species
            ui.horizontal(|ui| {
                ui.label("Species:");
                ui.selectable_value(&mut spawn_settings.species, Species::EColi, "E. coli");
                ui.selectable_value(&mut spawn_settings.species, Species::Listeria, "Listeria");
            });

            // Select Count
            ui.horizontal(|ui| {
                ui.label("Count:  ");
                ui.add(egui::Slider::new(&mut spawn_settings.count, 1..=1000)
                    .show_value(true));
            });

            // Click spawn toggle
            ui.checkbox(&mut spawn_settings.click_spawn, "Spawn on Viewport Click");

            // Spawn randomly now button
            if ui.button("Spawn Randomly Now").clicked() {
                let mut rng = rand::thread_rng();
                for _ in 0..spawn_settings.count {
                    let pos = Vec2::new(
                        rng.gen_range(-300.0..300.0),
                        rng.gen_range(-300.0..300.0),
                    );
                    let dna = match spawn_settings.species {
                        Species::EColi => Dna::e_coli(),
                        Species::Listeria => Dna::listeria(),
                    };
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
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // --- SECTION 2: METRICS & STATISTICS ---
            ui.label(egui::RichText::new("SPECIES TELEMETRY").strong().color(egui::Color32::from_rgb(0, 200, 255)));
            ui.add_space(5.0);

            let mut ecoli_count = 0;
            let mut listeria_count = 0;
            let mut ecoli_stress = 0.0;
            let mut listeria_stress = 0.0;
            let mut max_generation = 0;

            for (dna, met) in bacteria_query.iter() {
                max_generation = max_generation.max(dna.generation);
                match dna.species {
                    Species::EColi => {
                        ecoli_count += 1;
                        ecoli_stress += met.accumulated_stress;
                    }
                    Species::Listeria => {
                        listeria_count += 1;
                        listeria_stress += met.accumulated_stress;
                    }
                }
            }

            let avg_ec_stress = if ecoli_count > 0 { (ecoli_stress / ecoli_count as f32).min(30.0) / 30.0 } else { 0.0 };
            let avg_lis_stress = if listeria_count > 0 { (listeria_stress / listeria_count as f32).min(30.0) / 30.0 } else { 0.0 };
            let food_count = food_query.iter().count();

            // E. coli row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("E. coli (Cyan):").color(egui::Color32::from_rgb(0, 200, 255)));
                ui.label(egui::RichText::new(format!("{}", ecoli_count)).strong());
            });
            ui.horizontal(|ui| {
                ui.label("  Avg Stress:");
                let color = egui::Color32::from_rgb(
                    (avg_ec_stress * 255.0) as u8,
                    ((1.0 - avg_ec_stress) * 255.0) as u8,
                    0
                );
                ui.add(egui::ProgressBar::new(avg_ec_stress)
                    .text(format!("{:.1}%", avg_ec_stress * 100.0))
                    .fill(color));
            });

            ui.add_space(5.0);

            // Listeria row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Listeria (Green):").color(egui::Color32::from_rgb(50, 255, 100)));
                ui.label(egui::RichText::new(format!("{}", listeria_count)).strong());
            });
            ui.horizontal(|ui| {
                ui.label("  Avg Stress:");
                let color = egui::Color32::from_rgb(
                    (avg_lis_stress * 255.0) as u8,
                    ((1.0 - avg_lis_stress) * 255.0) as u8,
                    0
                );
                ui.add(egui::ProgressBar::new(avg_lis_stress)
                    .text(format!("{:.1}%", avg_lis_stress * 100.0))
                    .fill(color));
            });

            ui.add_space(5.0);
            ui.label(format!("Nutrients Level: {}", food_count));
            ui.label(format!("Max Generation Reached: {}", max_generation));

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // --- SECTION 3: REAL-TIME POPULATION GRAPH ---
            ui.label(egui::RichText::new("POPULATION PLOT").strong().color(egui::Color32::from_rgb(0, 200, 255)));
            ui.add_space(5.0);

            // Drawing a custom scientific chart inside the panel using Egui canvas painter
            let size = egui::Vec2::new(240.0, 120.0);
            let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            
            // Draw background grid box
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(15, 20, 30));
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 70)));

            // Draw axis lines
            let max_val = history.ecoli_counts.iter().copied()
                .chain(history.listeria_counts.iter().copied())
                .fold(100.0, f32::max); // Minimum scale limit

            let len = history.ecoli_counts.len();
            if len > 1 {
                let dx = rect.width() / (history.max_history as f32);
                let dy = rect.height() / max_val;

                // Helper lambda to map index and value to canvas pixels
                let map_point = |idx: usize, val: f32| -> egui::Pos2 {
                    let x = rect.left() + (idx as f32) * dx;
                    let y = rect.bottom() - val * dy;
                    egui::Pos2::new(x, y.clamp(rect.top() + 5.0, rect.bottom()))
                };

                // Draw E. coli line (Cyan)
                for i in 0..(len - 1) {
                    let p1 = map_point(i, history.ecoli_counts[i]);
                    let p2 = map_point(i + 1, history.ecoli_counts[i + 1]);
                    painter.line_segment([p1, p2], egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 200, 255)));
                }

                // Draw Listeria line (Green)
                for i in 0..(len - 1) {
                    let p1 = map_point(i, history.listeria_counts[i]);
                    let p2 = map_point(i + 1, history.listeria_counts[i + 1]);
                    painter.line_segment([p1, p2], egui::Stroke::new(1.5, egui::Color32::from_rgb(50, 255, 100)));
                }
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Accumulating telemetry...",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(100, 120, 140)
                );
            }

            ui.add_space(15.0);
            
            // Hazard flashing overlay
            if env.base_toxin_level > 0.0 {
                let flash = (ctx.input(|i| i.time) * 10.0).sin().abs();
                let bg_color = egui::Color32::from_rgba_unmultiplied(
                    200, 
                    (50.0 * flash) as u8, 
                    (50.0 * flash) as u8, 
                    180
                );
                
                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(5.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("WARNING: TOXIC CAGE BREACHED")
                                .color(egui::Color32::WHITE)
                                .strong());
                        });
                    });
            }
        });
}

/// System to spawn bacteria when clicking on the viewport.
pub fn handle_mouse_clicks(
    mut commands: Commands,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    spawn_settings: Res<SpawnSettings>,
    mut contexts: EguiContexts,
) {
    // If egui wants pointer input (e.g. user clicked on the panel), do not spawn!
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }

    if spawn_settings.click_spawn && mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(window) = window_query.get_single() {
            if let Ok((camera, camera_transform)) = camera_query.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        let mut rng = rand::thread_rng();
                        for _ in 0..spawn_settings.count {
                            // Add a tiny random offset around the click so they don't spawn on the exact same pixel
                            let offset = Vec2::new(
                                rng.gen_range(-10.0..10.0),
                                rng.gen_range(-10.0..10.0),
                            );
                            let pos = world_pos + offset;
                            
                            // Keep within bounds
                            let box_limit = 300.0;
                            let pos = Vec2::new(
                                pos.x.clamp(-box_limit, box_limit),
                                pos.y.clamp(-box_limit, box_limit),
                            );

                            let dna = match spawn_settings.species {
                                Species::EColi => Dna::e_coli(),
                                Species::Listeria => Dna::listeria(),
                            };
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
                    }
                }
            }
        }
    }
}
