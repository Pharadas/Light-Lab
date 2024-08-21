use std::f32::consts::PI;

use eframe::glow::Context;
use egui::{self, epaint::image, include_image, Button, Color32, ColorImage, Image, ImageSource, RichText, Slider, TextureHandle, TextureOptions, Ui};
use ::image::{ImageBuffer, Rgba, RgbImage, RgbaImage};
use math_vector::Vector;
use egui_plot::{Line, Plot, PlotItem, PlotPoints};
use web_sys::console;
use egui_extras::{TableBuilder, Column, RetainedImage};

use crate::world::{ObjectType, OpticalObject, PolarizerType, World, WorldObject};

pub struct MenusState {
    selected_object: OpticalObject,
    selected_polarizer_type: PolarizerType,
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
            angle: 0f32,
            relative_phase_retardation: 0f32,
            circularity: 0f32,
            object_creation_state: WorldObject::new(ObjectType::CubeWall),
            image_texture: texture,
            raw_images,
            image_sizes
        };
    }

    pub fn inspect_object_menu(&mut self, ui: &mut Ui, world: &mut World) {
        let sin: PlotPoints = (-100..100).map(|i| {
            let x = i as f64 * 0.01;
            [x, x.sin()]
        }).collect();
        let line = Line::new(sin).color(Color32::from_rgb(255, 0, 0));

        let neg_sin: PlotPoints = (-100..100).map(|i| {
            let x = i as f64 * 0.01;
            [x, -x.sin()]
        }).collect();
        let neg_line = Line::new(neg_sin).color(Color32::from_rgb(255, 0, 0));

        Plot::new("my_plot").allow_drag(false).allow_boxed_zoom(false).allow_zoom(false).view_aspect(2.0).show(ui, |plot_ui| {
            console::log_1(&format!("{:?}", plot_ui.pointer_coordinate_drag_delta()).into());
            plot_ui.line(line);
            plot_ui.line(neg_line);
        });
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

                let curr_image = self.raw_images[self.selected_polarizer_type as usize].clone();

                self.image_texture.set(
                    ColorImage::from_rgba_unmultiplied(self.image_sizes[self.selected_polarizer_type as usize], &curr_image.into_raw()),
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

                ui.add(
                    egui::Image::new(&self.image_texture)
                        .max_height(400.0)
                        .max_width(500.0)
                        .fit_to_exact_size(egui::Vec2 { x: 500.0, y: 500.0 })
                        .maintain_aspect_ratio(true)
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
