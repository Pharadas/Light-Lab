use std::{f32::consts::PI, ops::Deref};

use egui::{self, Button, Color32, ColorImage, Label, Shape, Slider, Stroke, TextureHandle, TextureOptions, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use ::image::{ImageBuffer, Rgba};
use egui_plot::Plot;
use nalgebra::{RealField, Vector3};
use web_sys::console;

use crate::{app::MainGlowProgram, world::{ObjectType, PolarizerType, World, WorldObject}};

pub struct MenusState {
    pub selected_object: Option<WorldObject>,
    selected_polarizer_type: PolarizerType,
    angle: f32,
    relative_phase_retardation: f32,
    circularity: f32,
    object_creation_state: WorldObject,
    image_texture: TextureHandle,
    debug_texture: TextureHandle,
    raw_images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    image_sizes: Vec<[usize; 2]>,
    pub should_display_debug_menu: bool,
    should_display_debug_objects_view: bool
}

// rand doesnt work good with wasm, so we will just generate them
fn generate_colors_list() -> Vec<[u8; 4]> {
    vec![
        [0, 0, 0, 255],
        [164,138,150, 255],
        [9,62,36, 255],
        [200,40,235, 255],
        [52,112,129, 255],
        [78,175,51, 255],
        [53,138,30, 255],
        [183,171,239, 255],
        [2,67,188, 255],
        [91,113,64, 255],
        [235,39,232, 255],
        [60,69,123, 255],
        [200,40,235, 255],
    ]
}

impl MenusState {
    pub fn new(image_texture: TextureHandle, debug_texture: TextureHandle, raw_images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>, image_sizes: Vec<[usize; 2]>) -> MenusState {
        return MenusState {
            selected_object: None,
            selected_polarizer_type: PolarizerType::LinearHorizontal,
            angle: 0f32,
            relative_phase_retardation: 0f32,
            circularity: 0f32,
            object_creation_state: WorldObject::new(),
            image_texture,
            debug_texture,
            raw_images,
            image_sizes,
            should_display_debug_menu: false,
            should_display_debug_objects_view: false
        };
    }

    pub fn debug_menu(&mut self, ui: &mut Ui, world: &mut World, glow_program: MainGlowProgram) {
        ui.add(Label::new(format!("Current resolution: {:?}", glow_program.current_texture_resolution)));

        egui::CollapsingHeader::new("Gpu compatible objects list")
            .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(Label::new(format!("{:?}", world.get_gpu_compatible_world_objects_list().chunks(21).into_iter().map(|chunk| chunk).collect::<Vec<&[u32]>>())));
            });
        });

        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Index");
                });
                header.col(|ui| {
                    ui.heading("World object");
                });
            })
            .body(|mut body| {
                for i in 0..world.objects.len() {
                    body.row(10.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{:?}", i));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:?}", world.objects[i]));
                        });
                    });
                }
            });

        if ui.add(Button::new("Print hashmap")).clicked() {
            console::log_1(&format!("{:?}", world.hash_map).into());
        }

        ui.add(Label::new(format!("Current object jones matrix: {:#?}", self.object_creation_state.jones_matrix)));

        if self.should_display_debug_objects_view {
            if ui.add(Button::new("Hide debug objects view")).clicked() {
                self.should_display_debug_objects_view = false;
            }

        } else {
            if ui.add(Button::new("Show debug objects view (WARNING, EXTREMELY SLOW)")).clicked() {
                self.should_display_debug_objects_view = true;
            }
        }

        if self.should_display_debug_objects_view {
            let colors = generate_colors_list();
            let objects_colored: Vec<u8> = glow_program.objects_found.chunks(4).flat_map(|x| colors[(x[0] % 12) as usize]).rev().collect();

            let curr_image = &objects_colored;

            self.debug_texture.set(
                ColorImage::from_rgba_unmultiplied([
                    glow_program.current_texture_resolution[0] as usize,
                    glow_program.current_texture_resolution[1] as usize
                ], &curr_image),
                TextureOptions::default(),
            );

            ui.add(
                egui::Image::new(&self.debug_texture)
                    .max_height(400.0)
                    .max_width(500.0)
            );
        }
    }

    pub fn inspect_object_menu(&mut self, ui: &mut Ui, world: &mut World, time: f64, selected_object_index: &mut usize) {
        ui.add(Label::new(format!("{:?}", world.objects[*selected_object_index])));

        if ui.add(Button::new("Remove object")).clicked() {
            world.remove_object(*selected_object_index);
            *selected_object_index = 0;
        }

        if (
            world.objects[*selected_object_index].object_type == ObjectType::SquareWall ||
            world.objects[*selected_object_index].object_type == ObjectType::RoundWall ||
            world.objects[*selected_object_index].object_type == ObjectType::OpticalObjectSquareWall ||
            world.objects[*selected_object_index].object_type == ObjectType::OpticalObjectRoundWall
        ) {
            let mut shapes = vec![];

            ui.add(Slider::new(&mut world.objects[*selected_object_index].rotation[0], 0.0..=2.0*PI).text("X rotation"));
            ui.add(Slider::new(&mut world.objects[*selected_object_index].rotation[1], 0.0..=2.0*PI).text("Y rotation"));
            ui.add(Slider::new(&mut world.objects[*selected_object_index].radius, 0.0..=2.0*PI).text("Radius"));

            let response = Plot::new("rotation_plot")
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .allow_zoom(false)
            .include_x(1.0)
            .include_y(1.0)
            .include_x(-1.0)
            .include_y(-1.0)
            .view_aspect(1.0)
            .show(ui, |plot_ui| {
                // vertical
                shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new((world.objects[*selected_object_index].rotation[0].abs() * 150.0) / (2.0 * PI), 150.0), Stroke::new(1.0, Color32::BLUE)));

                // horizontal
                shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(150.0, (world.objects[*selected_object_index].rotation[1].abs() * 150.0) / (2.0 * PI)), Stroke::new(1.0, Color32::GREEN)));

                world.objects[*selected_object_index].rotation[0] += plot_ui.pointer_coordinate_drag_delta().x * 2.0;
                world.objects[*selected_object_index].rotation[1] += plot_ui.pointer_coordinate_drag_delta().y * 2.0;

                world.objects[*selected_object_index].rotation[0] = world.objects[*selected_object_index].rotation[0].clamp(0.0, 2.0 * PI);
                world.objects[*selected_object_index].rotation[1] = world.objects[*selected_object_index].rotation[1].clamp(0.0, 2.0 * PI);
            }).response;

            ui.painter().with_clip_rect(response.rect).extend(shapes);
        }
    }

    pub fn object_creation_menu(&mut self, ui: &mut Ui, world: &mut World, viewer_position: &Vector3<f32>) {
        egui::ComboBox::from_label("Polarizer/Phase retarder")
            .selected_text(format!("{}", self.object_creation_state.object_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::CubeWall,                "Wall (cube)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::SquareWall,              "Wall (square)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::RoundWall,               "Wall (round)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::LightSource,             "Light source (sphere)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::OpticalObjectCube,       "Optical object (cube)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::OpticalObjectSquareWall, "Optical object (square)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::OpticalObjectRoundWall,  "Optical object (round)");
            }
        );

        ui.add_space(10.0);

        match self.object_creation_state.object_type {
            ObjectType::LightSource => {
                self.object_creation_state.center = [viewer_position.x, viewer_position.y, viewer_position.z];
                self.object_creation_state.radius = 0.5;
            }

            ObjectType::OpticalObjectCube       |
            ObjectType::OpticalObjectRoundWall  |
            ObjectType::OpticalObjectSquareWall
                => {
                egui::ComboBox::from_label("Type of polarizer/phase retarder")
                    .selected_text(format!("{}", self.selected_polarizer_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::LinearHorizontal, "Linear horizontal");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::LinearVertical, "Linear vertical");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::Linear45Degrees, "Linear rotated 45 degrees");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::LinearTheta, "Linear rotated θ degrees");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::RightCircular, "Right circular");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::LeftCircular, "Left circular");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::QuarterWavePlateFastAxisVertical, "Quarter-wave plate with fast axis vertical");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::QuarterWavePlateFastAxisHorizontal, "Quarter-wave plate with fast axis horizontal");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::QuarterWavePlateFastAxisTheta, "Quarter-wave plate with fast axis at angle θ w.r.t the horizontal axis ");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::HalfWavePlateRotatedTheta, "Half-wave plate rotated by θ");
                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::HalfWavePlateFastAxisTheta, "Half-wave plate with fast axis at angle θ w.r.t the horizontal axis");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::GeneralWavePlateLinearRetarderTheta, "General Waveplate (Linear Phase Retarder)");

                        ui.selectable_value(&mut self.selected_polarizer_type, PolarizerType::ArbitraryBirefringentMaterialTheta, "Arbitrary birefringent material (Elliptical phase retarder)");
                    }
                );

                let curr_image = &self.raw_images[self.selected_polarizer_type as usize];

                self.image_texture.set(
                    ColorImage::from_rgba_unmultiplied(self.image_sizes[self.selected_polarizer_type as usize], &curr_image),
                    TextureOptions::default(),
                );

                ui.add_space(10.0);

                match self.selected_polarizer_type {
                    PolarizerType::LinearTheta                   | 
                    PolarizerType::QuarterWavePlateFastAxisTheta | 
                    PolarizerType::HalfWavePlateFastAxisTheta    | 
                    PolarizerType::HalfWavePlateRotatedTheta     => {
                        ui.add(Slider::new(&mut self.angle, 0.0..=2.0*PI).text("θ"));
                    }

                    PolarizerType::GeneralWavePlateLinearRetarderTheta => {
                        ui.add(Slider::new(&mut self.angle, 0.0..=PI).text("θ"));
                        ui.add(Slider::new(&mut self.relative_phase_retardation, 0.0..=2.0*PI).text("Relative phase retardation (η)"));
                    }

                    PolarizerType::ArbitraryBirefringentMaterialTheta => {
                        ui.add(Slider::new(&mut self.angle, 0.0..=PI).text("θ"));
                        ui.add(Slider::new(&mut self.relative_phase_retardation, 0.0..=2.0*PI).text("Relative phase retardation (η)"));
                        ui.add(Slider::new(&mut self.circularity, (-PI/2.0)..=(PI/2.0)).text("Circularity (φ)"));
                    }

                    _ => {}
                }

                self.object_creation_state.set_jones_matrix(self.selected_polarizer_type, self.angle, self.relative_phase_retardation, self.circularity);

                ui.add_space(10.0);

                ui.add(
                    egui::Image::new(&self.image_texture)
                        .max_height(400.0)
                        .max_width(500.0)
                        // .fit_to_exact_size(egui::Vec2 { x: 500.0, y: 500.0 })
                        // .maintain_aspect_ratio(true)
                );

                ui.add_space(10.0);
            }

            ObjectType::CubeWall   |
            ObjectType::RoundWall  |
            ObjectType::SquareWall
                => {
                self.object_creation_state.center = [viewer_position.x, viewer_position.y, viewer_position.z];
                self.object_creation_state.radius = 0.5;
            }
        }

        if ui.add(Button::new("Create object in your position")).clicked() {
            world.insert_object(Vector3::from_vec(viewer_position.as_slice().into_iter().map(|x| *x as i32).collect()), self.object_creation_state.clone());
        }
    }
}
