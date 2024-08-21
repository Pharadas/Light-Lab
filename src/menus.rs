use std::f32::consts::PI;

use eframe::glow::Context;
use egui::{self, epaint::image, include_image, Button, Color32, ColorImage, Image, ImageSource, Pos2, Response, RichText, Shape, Slider, Stroke, TextureHandle, TextureOptions, Ui, Vec2};
use ::image::{ImageBuffer, Rgba, RgbImage, RgbaImage};
use math_vector::Vector;
use egui_plot::{Line, Plot, PlotItem, PlotPoints, PlotResponse};
use nalgebra::ComplexField;
use web_sys::console;
use egui_extras::{TableBuilder, Column, RetainedImage};

use crate::world::{ObjectType, OpticalObject, PolarizerType, World, WorldObject};

pub struct MenusState {
    selected_object: OpticalObject,
    selected_polarizer_type: PolarizerType,
    rotation: Vec2,
    angle: f32,
    relative_phase_retardation: f32,
    circularity: f32,
    object_creation_state: WorldObject,
    image_texture: TextureHandle,
    raw_images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    image_sizes: Vec<[usize; 2]>
}

impl MenusState {
    pub fn new(texture: TextureHandle, raw_images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>, image_sizes: Vec<[usize; 2]>) -> MenusState {
        return MenusState {
            selected_object: OpticalObject::LightSource,
            selected_polarizer_type: PolarizerType::LinearHorizontal,
            rotation: Vec2::new(150.0, 150.0),
            angle: 0f32,
            relative_phase_retardation: 0f32,
            circularity: 0f32,
            object_creation_state: WorldObject::new(ObjectType::CubeWall),
            image_texture: texture,
            raw_images,
            image_sizes
        };
    }

    pub fn debug_menu(&mut self, ui: &mut Ui, world: &mut World) {
        if ui.add(Button::new("Print hashmap")).clicked() {
            console::log_1(&format!("{:?}", world.hash_map).into());
        }
    }

    pub fn inspect_object_menu(&mut self, ui: &mut Ui, world: &mut World, time: f64) {
        let a_vertical = 0.01 * time;

        let vertical_ellipse_top_half: PlotPoints = (-100..=100).map(|i| {
            let x = i as f64 * 0.01;
            [x, (1.0 / a_vertical) * (a_vertical.powi(2) - x.powi(2)).sqrt()]
        }).collect();

        // let vertical_ellipse_bottom_half: PlotPoints = (-100..=100).map(|i| {
        //     let x = i as f64 * 0.01;
        //     [x,-(1.0 / a_vertical) * (a_vertical.powi(2) - x.powi(2)).sqrt()]
        // }).collect();
        // let ellipse_bottom_half = Line::new(vertical_ellipse_bottom_half).color(Color32::from_rgb(255, 0, 0));

        let mut shapes = vec![];

        ui.add(Slider::new(&mut self.rotation.x, -150.0..=150.0).text("X rotation"));
        ui.add(Slider::new(&mut self.rotation.y, -150.0..=150.0).text("Y rotation"));

        let response = Plot::new("my_plot")
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
            // shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(1.0, 150.0), Stroke::new(1.0, Color32::from_rgb(255, 0, 0))));
            shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(self.rotation.x.abs(), 150.0), Stroke::new(1.0, Color32::BLUE)));

            // horizontal
            // shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(150.0, 1.0), Stroke::new(1.0, Color32::from_rgb(255, 0, 0))));
            shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(150.0, self.rotation.y.abs()), Stroke::new(1.0, Color32::GREEN)));

            // shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), self.rotation.abs(), Stroke::new(1.0, Color32::from_rgb(255, 0, 0))));

            self.rotation += plot_ui.pointer_coordinate_drag_delta() * 20.0;
            // console::log_1(&format!("{:?}", plot_ui.pointer_coordinate_drag_delta()).into());
            // plot_ui.line(ellipse_top_half);
            // plot_ui.line(ellipse_bottom_half);
        }).response;

        ui.painter().with_clip_rect(response.rect).extend(shapes);
    }

    pub fn select_object_menu(&mut self, ui: &mut Ui, world: &mut World, viewer_position: &Vector<f32>) {
        egui::ComboBox::from_label("Polarizer/Phase retarder")
            .selected_text(format!("{}", self.selected_object))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected_object, OpticalObject::LightSource, "Light source");
                ui.selectable_value(&mut self.selected_object, OpticalObject::Polarizer_PhaseRetarder, "Polarizer/Phase retarder");
                ui.selectable_value(&mut self.selected_object, OpticalObject::Wall, "Wall");
            }
        );

        ui.add_space(10.0);

        match self.selected_object {
            OpticalObject::LightSource => {
                // Polarization, color
            }

            OpticalObject::Polarizer_PhaseRetarder => {
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
                        ui.add(Slider::new(&mut self.angle, 0.0..=2.0*PI).text("θ"));
                        ui.add(Slider::new(&mut self.relative_phase_retardation, 0.0..=2.0*PI).text("Relative phase retardation (η)"));
                    }

                    PolarizerType::ArbitraryBirefringentMaterialTheta => {
                        ui.add(Slider::new(&mut self.angle, 0.0..=2.0*PI).text("θ"));
                        ui.add(Slider::new(&mut self.relative_phase_retardation, 0.0..=2.0*PI).text("Relative phase retardation (η)"));
                        ui.add(Slider::new(&mut self.circularity, (-PI/2.0)..=(PI/2.0)).text("Circularity (φ)"));
                    }

                    _ => {}
                }

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

            OpticalObject::Wall => {
                // position, color
            }

            _ => {}
        }

        if ui.add(Button::new("Crear objeto sobre tu posicion")).clicked() {
            world.insert_object(viewer_position.as_i32s(), self.object_creation_state.clone());
        }
    }
}
