use std::f32::consts::PI;

use egui::{self, Button, Color32, ColorImage, Label, Shape, Slider, Stroke, TextureHandle, TextureOptions, Ui, Vec2};
use ::image::{ImageBuffer, Rgba};
use math_vector::Vector;
use egui_plot::{Plot, PlotPoints};
use web_sys::console;

use crate::{app::MainGlowProgram, world::{ObjectType, OpticalObject, PolarizerType, World, WorldObject}};

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
            image_sizes,
            should_display_debug_menu: false,
            should_display_debug_objects_view: false
        };
    }

    pub fn debug_menu(&mut self, ui: &mut Ui, world: &mut World, glow_program: MainGlowProgram) {
        ui.add(Label::new(format!("{:?}", glow_program.current_texture_resolution)));

        if ui.add(Button::new("Print hashmap")).clicked() {
            console::log_1(&format!("{:?}", world.hash_map).into());
        }

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

            self.image_texture.set(
                ColorImage::from_rgba_unmultiplied([
                    glow_program.current_texture_resolution[0] as usize,
                    glow_program.current_texture_resolution[1] as usize
                ], &curr_image),
                TextureOptions::default(),
            );

            ui.add(
                egui::Image::new(&self.image_texture)
                    .max_height(400.0)
                    .max_width(500.0)
            );
        }
    }

    pub fn inspect_object_menu(&mut self, ui: &mut Ui, world: &mut World, time: f64) {
        let a_vertical = 0.01 * time;

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
