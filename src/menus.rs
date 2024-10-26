use std::f32::consts::PI;

use egui::{self, color_picker::color_picker_color32, Button, Color32, ColorImage, Label, Shape, Slider, Stroke, TextureHandle, TextureOptions, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use ::image::{ImageBuffer, Rgba};
use egui_plot::{Line, Plot, PlotPoints};
use nalgebra::{Complex, ComplexField, Vector2, Vector3};
use web_sys::console;

use crate::{app::MainGlowProgram, camera::{rotate3d_x, rotate3d_y}, demos::{coordinated_interference_demo, double_slit_demo, light_profile, no_demo, simple_interference_demo, triple_slit_demo, uncoordinated_interference_demo, Demo}, world::{Alignment, LightPolarizationType, ObjectType, PolarizerType, World, WorldObject}};

pub struct MenusState {
    pub selected_demo: Demo, 
    last_selected_demo: Demo, 

    selected_polarizer_type: PolarizerType,
    selected_light_polarization: LightPolarizationType,
    selected_color: Color32,
    angle: f32,
    relative_phase_retardation: f32,
    circularity: f32,
    object_creation_state: WorldObject,
    image_texture: TextureHandle,
    debug_texture: TextureHandle,
    raw_images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    image_sizes: Vec<[usize; 2]>,
    pub should_display_debug_menu: bool,
    pub trying_to_align_to_object: bool,
    should_display_debug_objects_view: bool
}

// rand doesnt work good with wasm, so we will just generate them
fn generate_colors_list() -> Vec<[u8; 4]> {
    vec![
        [250, 100, 0, 255],
        [164,250,150, 255],
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
            selected_demo: Demo::None,
            last_selected_demo: Demo::None,

            selected_polarizer_type: PolarizerType::LinearHorizontal,
            selected_color: Color32::from_rgb(250, 50, 250),
            selected_light_polarization: LightPolarizationType::LinearHorizontal,
            angle: 0f32,
            relative_phase_retardation: 0f32,
            circularity: 0f32,
            object_creation_state: WorldObject::new(),
            image_texture,
            debug_texture,
            raw_images,
            image_sizes,
            should_display_debug_menu: false,
            trying_to_align_to_object: false,
            should_display_debug_objects_view: false
        };
    }

    pub fn debug_menu(&mut self, ui: &mut Ui, world: &mut World, glow_program: MainGlowProgram) {
        ui.add(Label::new(format!("Current resolution: {:?}", glow_program.current_texture_resolution)));

        egui::CollapsingHeader::new("Gpu compatible objects list")
            .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(Label::new(format!("{:?}", world.get_gpu_compatible_world_objects_list().chunks(24).into_iter().map(|chunk| chunk).collect::<Vec<&[u32]>>())));
            });
        });

        ui.add(Label::new(format!("Light sources: {:?}", world.light_sources)));
        ui.add(Label::new(format!("World objects stack: {:?}", world.objects_stack)));
        ui.add(Label::new(format!("World objects associations: {:?}", world.objects_associations)));
        ui.add(Label::new(format!("Aligned objects: {:?}", world.aligned_objects)));

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
        ui.add(Label::new(format!("{:?}", world.objects[*selected_object_index].object_type)));
        ui.add(Label::new(format!("Object index: {:?}", *selected_object_index)));

        if self.trying_to_align_to_object && ui.add(Button::new("Cancel object align")).clicked() {
            self.trying_to_align_to_object = false;

        } else if world.objects[*selected_object_index].aligned_to_object != 0 {
            ui.add(Label::new(format!("Aligned to: {:?}", world.objects[world.objects[*selected_object_index].aligned_to_object].object_type)));

            egui::ComboBox::from_label("Object alignment")
                .selected_text(format!("{}", world.objects[*selected_object_index].alignment))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut world.objects[*selected_object_index].alignment, Alignment::FRONT, "Front");
                    ui.selectable_value(&mut world.objects[*selected_object_index].alignment, Alignment::RIGHT, "Right");
                    ui.selectable_value(&mut world.objects[*selected_object_index].alignment, Alignment::UP, "Up");
                }
            );

            ui.add(Slider::new(&mut world.objects[*selected_object_index].aligned_distance, -1.0..=1.0).text("Distance from object"));

            if ui.add(Button::new("Remove alignment")).clicked() {
                world.objects[*selected_object_index].aligned_to_object = 0;
                world.objects[*selected_object_index].alignment = Alignment::FRONT;
                world.objects[*selected_object_index].aligned_distance = 0.0;
            }

        } else if ui.add(Button::new("Align to object")).clicked() {
            self.trying_to_align_to_object = true;
        }

        if ui.add(Button::new("Remove object")).clicked() {
            world.remove_object(*selected_object_index);
            *selected_object_index = 0;
            return;
        }

        let mut shapes = vec![];

        ui.add(Slider::new(&mut world.objects[*selected_object_index].radius, 0.05..=0.5).text("Object size"));

        ui.add(Slider::new(&mut world.objects[*selected_object_index].rotation[0], -PI..=PI).text("X rotation"));
        ui.add(Slider::new(&mut world.objects[*selected_object_index].rotation[1], -PI..=PI).text("Y rotation"));

        ui.label("Drag to rotate object!");

        let response = Plot::new("rotation_plot")
        .allow_drag(false)
        .allow_boxed_zoom(false)
        .allow_zoom(false)
        .include_x(1.0)
        .include_y(1.0)
        .include_x(-1.0)
        .include_y(-1.0)
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            // vertical
            shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new((world.objects[*selected_object_index].rotation[0].abs() * 100.0) / PI, 100.0), Stroke::new(1.0, Color32::BLUE)));

            // horizontal
            shapes.push(Shape::ellipse_stroke(plot_ui.screen_from_plot([0.0, 0.0].into()), Vec2::new(100.0, (world.objects[*selected_object_index].rotation[1].abs() * 100.0) / PI), Stroke::new(1.0, Color32::GREEN)));

            world.objects[*selected_object_index].rotation[0] += plot_ui.pointer_coordinate_drag_delta().x * 2.0;
            world.objects[*selected_object_index].rotation[1] += plot_ui.pointer_coordinate_drag_delta().y * 2.0;

            world.objects[*selected_object_index].rotation[0] = world.objects[*selected_object_index].rotation[0].clamp(-PI, PI);
            world.objects[*selected_object_index].rotation[1] = world.objects[*selected_object_index].rotation[1].clamp(-PI, PI);
        }).response;

        ui.painter().with_clip_rect(response.rect).extend(shapes);

        match world.objects[*selected_object_index].object_type {
            ObjectType::LightSource => {
                ui.add(Slider::new(&mut world.objects[*selected_object_index].wavelength, 0.001..=1.0).text("Wavelength"));
                ui.add(Label::new("Light polarization"));

                egui::ComboBox::from_label("Light source polarization")
                    .selected_text(format!("{}", world.objects[*selected_object_index].polarization_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::LinearHorizontal, "Linear horizontal");
                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::LinearVertical, "Linear vertical");

                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::LinearDiagonal, "Linear rotated 45 degrees");
                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::LinearAntiDiagonal, "Linear rotated -45 degrees");

                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::CircularRightHand, "Right circular");
                        ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::CircularLeftHand, "Left circular");

                        // ui.selectable_value(&mut world.objects[*selected_object_index].polarization_type, LightPolarizationType::NotPolarized, "Not polarized");
                    }
                );

                world.objects[*selected_object_index].set_light_polarization();

                let retardation = Vector2::new(0.0, 0.0);
                let angular_frequency = 1.0;

                let jones_vector: Vector2<Complex<f32>> = Vector2::new(
                    (Complex::new(0.0f32, 1.0f32 * retardation.x)).exp(),
                    (Complex::new(0.0f32, 1.0f32 * retardation.y)).exp(),
                ) * Complex::new(0.0f32, (-angular_frequency * time) as f32).exp();

                let final_jones_vector = [0, 1].map(|i| jones_vector[i] * world.objects[*selected_object_index].polarization[i]);

                let real: PlotPoints = (0..1000).map(|i| {
                    [(final_jones_vector[0] * 0.001 * i as f32).real() as f64, (final_jones_vector[1] * 0.001 * i as f32).real() as f64]
                }).collect();

                let real_line = Line::new(real);

                let imaginary: PlotPoints = (0..1000).map(|i| {
                    [(final_jones_vector[0] * 0.001 * i as f32).imaginary() as f64, (final_jones_vector[1] * 0.001 * i as f32).imaginary() as f64]
                }).collect();

                let imaginary_line = Line::new(imaginary);

                Plot::new("my_plot")
                    .view_aspect(2.0)
                    .include_x(1.0)
                    .include_y(1.0)
                    .include_x(-1.0)
                    .include_y(-1.0)
                    .show(ui, |plot_ui| {plot_ui.line(real_line); plot_ui.line(imaginary_line)});
            }
            ObjectType::CubeWall => todo!(),
            ObjectType::SquareWall => todo!(),
            ObjectType::RoundWall => {}
            ObjectType::OpticalObjectCube => {}
            ObjectType::OpticalObjectSquareWall => todo!(),
            ObjectType::OpticalObjectRoundWall => {}
        }

        color_picker_color32(ui, &mut world.objects[*selected_object_index].color, egui::color_picker::Alpha::Opaque);
    }

    pub fn info_menu(&mut self, ui: &mut Ui) {
        match self.selected_demo {
            Demo::None => {
                ui.label("This is a mini optics laboratory, it's main purpose is to allow everyone to understand light polarization in an intuitive and visual way");
                ui.add_space(4.0);

                ui.label("WASD to move, drag on the window to look around, click on an object you created to select it and align them to get the interference pattern just right");
                ui.add_space(4.0);

                ui.label("Try creating light sources with multiple different polarizations and see how they change across space, try creating interference and adding polarizers and phase retarders and see how they interact!");
                ui.add_space(4.0);

                ui.label("Please have fun and if anything breaks, you can raise an issue on the github repo");
                ui.add_space(4.0);

                ui.hyperlink("https://github.com/Pharadas/Light-Lab");
            }

            Demo::LightProfile => {
                ui.label("This demo shows the profile of the every light source you create, try clicking on it and rotating it");
                ui.add_space(4.0);

                ui.label("The light is modeled as a gaussian beam (a laser light)");
                ui.hyperlink("https://en.wikipedia.org/wiki/Gaussian_beam");
            }

            Demo::SimpleInterferenceDemo => {
                ui.label("This demo shows an emulation of what a michaelson interferometer attempts to do, light sources in real life never share a polarization and wavelength, so with an interferometer we split a light source so that two beams have the same properties, here we can simply create two light sources with the same properties");
                ui.add_space(4.0);

                ui.label("Try selecting the red light source and moving the 'Distance from object' slider, see how it changes!");
                ui.add_space(4.0);

                ui.label("Also try creating new light sources changing their wavelengths and playing with the slider 'Cube size in meters' to see how this pattern develops as you get closer or farther from the light source");
                ui.add_space(4.0);

                ui.hyperlink("https://en.wikipedia.org/wiki/Michelson_interferometer");
                ui.add_space(4.0);

                ui.label("In this demo we are simulating configuration a:");
                ui.hyperlink("https://upload.wikimedia.org/wikipedia/commons/0/01/Michelson_interferometer_fringe_formation.svg");
            }

            Demo::DoubleSlit => {
                ui.label("This demo shows an approximation of the double slit experiment, in this experiment we make a single light source interact with itself, here we can simply create two light sources with the same properties and make them interact");
                ui.add_space(4.0);

                ui.label("Try selecting the red light source and moving the 'Distance from object' slider, see how it changes!");
                ui.add_space(4.0);

                ui.label("Also try creating new light sources changing their wavelengths and playing with the slider 'Cube size in meters' to see how this pattern develops as you get closer or farther from the light source");
                ui.add_space(4.0);

                ui.hyperlink("https://en.wikipedia.org/wiki/Double-slit_experiment");
                ui.add_space(4.0);

                ui.label("We are simulating the bottom set of images");
                ui.hyperlink("https://www.researchgate.net/publication/335957159/figure/fig2/AS:805377650204672@1569028403287/a-d-Interference-patterns-of-the-light-passing-through-the-axicon-interfering-with-the.ppm");
                ui.add_space(4.0);
            }

            Demo::TripleSlit => {
                ui.label("This demo shows an approximation of the triple slit experiment, in this experiment we make a single light source interact with itself, here we can simply create three light sources with the same properties and make them interact");
                ui.add_space(4.0);

                ui.label("Try selecting the red light source and moving the 'Distance from object' slider, see how it changes!");
                ui.add_space(4.0);

                ui.label("Also try creating new light sources changing their wavelengths and playing with the slider 'Cube size in meters' to see how this pattern develops as you get closer or farther from the light source");
                ui.add_space(4.0);

                ui.hyperlink("https://en.wikipedia.org/wiki/Double-slit_experiment");
                ui.add_space(4.0);

                ui.label("Here is an example of what we are trying to simulate, note that the profile seen in the experiment is precisely what we get in this visualization");
                ui.hyperlink("https://www.researchgate.net/publication/263025761/figure/fig1/AS:324895427842053@1454472514561/A-schematic-diagram-of-the-three-slit-interference-experiment-with-a-quantum-which-path.png");
                ui.add_space(4.0);
            }

            Demo::UncoordinatedInterference => {
                ui.label("This experiment demonstrates many light sources with the same polarization but all out of phase with one another");
                ui.add_space(4.0);

                ui.label("Try creating new light sources and changing their polarizations to see how they interact with the others");
                ui.add_space(4.0);

                ui.label("Also try changing their wavelengths and playing with the slider 'Cube size in meters' to see how this pattern develops as you get closer or farther from the light source");
                ui.add_space(4.0);
            }

            Demo::CoordinatedInterference => {
                ui.label("This experiment demonstrates many light sources with the same polarization all in of phase with one another");
                ui.add_space(4.0);

                ui.label("Try creating new light sources and changing their polarizations to see how they interact with the others");
                ui.add_space(4.0);

                ui.label("Try deleting a few lights to create a square or a diamond");
                ui.add_space(4.0);

                ui.label("Here is an image from a real experiment of the same effect");
                ui.hyperlink("https://atoptics.co.uk/img/blog/venus-diffraction-gratings-opod-1.png");
            }
        }
    }

    pub fn select_demo(&mut self, ui: &mut Ui, world: &mut World, glow: &mut MainGlowProgram) {
        egui::ComboBox::from_label("Selected demo")
            .selected_text(format!("{}", self.selected_demo))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected_demo, Demo::None, "No demo");
                ui.selectable_value(&mut self.selected_demo, Demo::LightProfile, "Light profile demo");
                ui.selectable_value(&mut self.selected_demo, Demo::SimpleInterferenceDemo, "Basic interference demo");
                ui.selectable_value(&mut self.selected_demo, Demo::DoubleSlit, "Double slit demo");
                ui.selectable_value(&mut self.selected_demo, Demo::TripleSlit, "Triple slit demo");
                ui.selectable_value(&mut self.selected_demo, Demo::UncoordinatedInterference, "Uncoordinated interference demo");
                ui.selectable_value(&mut self.selected_demo, Demo::CoordinatedInterference, "Coordinated interference demo");
            }
        );

        if self.selected_demo != self.last_selected_demo {
            let demo_world;

            match self.selected_demo {
                Demo::None => {
                    demo_world = no_demo();
                    glow.cube_scaling_factor = 1.0;
                }

                Demo::LightProfile => {
                    demo_world = light_profile();
                    glow.cube_scaling_factor = 1.0;
                }

                Demo::SimpleInterferenceDemo => {
                    demo_world = simple_interference_demo();
                    glow.cube_scaling_factor = 1.0;
                }

                Demo::DoubleSlit => {
                    demo_world = double_slit_demo();
                    glow.cube_scaling_factor = 1.0;
                }


                Demo::TripleSlit => {
                    demo_world = triple_slit_demo();
                    glow.cube_scaling_factor = 1.0;
                }

                Demo::UncoordinatedInterference => {
                    demo_world = uncoordinated_interference_demo();
                    glow.cube_scaling_factor = 1.0;
                }

                Demo::CoordinatedInterference => {
                    demo_world = coordinated_interference_demo();
                    glow.cube_scaling_factor = 1.0;
                }
            }

            *world = demo_world;
            self.last_selected_demo = self.selected_demo;
        }
    }

    pub fn object_creation_menu(&mut self, ui: &mut Ui, world: &mut World, viewer_position: Vector3<f32>, viewer_look_at_direction: Vector2<f32>) {
        egui::ComboBox::from_label("Polarizer/Phase retarder")
            .selected_text(format!("{}", self.object_creation_state.object_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::RoundWall,               "Wall (round)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::LightSource,             "Light source (sphere)");
                ui.selectable_value(&mut self.object_creation_state.object_type, ObjectType::OpticalObjectRoundWall,  "Optical object (round)");
            }
        );

        ui.add_space(10.0);

        color_picker_color32(ui, &mut self.selected_color, egui::color_picker::Alpha::Opaque);

        ui.add_space(10.0);

        match self.object_creation_state.object_type {
            ObjectType::LightSource => {
                self.object_creation_state.center = [viewer_position.x, viewer_position.y, viewer_position.z];
                self.object_creation_state.radius = 0.1;

                egui::ComboBox::from_label("Light source polarization")
                    .selected_text(format!("{}", self.selected_light_polarization))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::LinearHorizontal, "Linear horizontal");
                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::LinearVertical, "Linear vertical");

                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::LinearDiagonal, "Linear rotated 45 degrees");
                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::LinearAntiDiagonal, "Linear rotated -45 degrees");

                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::CircularRightHand, "Right circular");
                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::CircularLeftHand, "Left circular");

                        ui.selectable_value(&mut self.selected_light_polarization, LightPolarizationType::NotPolarized, "Not polarized");
                    }
                );

                // // TODO: add new images at the end of this array and just add 12 to the value
                // let curr_image = &self.raw_images[self.selected_light_polarization as usize];

                // self.image_texture.set(
                //     ColorImage::from_rgba_unmultiplied(self.image_sizes[self.selected_light_polarization as usize], &curr_image),
                //     TextureOptions::default(),
                // );

               ui.add_space(10.0);

                self.object_creation_state.polarization_type = self.selected_light_polarization;
                self.object_creation_state.set_light_polarization();

                // ui.add_space(10.0);

                // ui.add(
                //     egui::Image::new(&self.image_texture)
                //         .max_height(400.0)
                //         .max_width(500.0)
                //         // .fit_to_exact_size(egui::Vec2 { x: 500.0, y: 500.0 })
                //         // .maintain_aspect_ratio(true)
                // );

                // ui.add_space(10.0);
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
            // we want to spawn the object a bit ahead from the viewer's look at direction
            let mut look_vector = Vector3::new(0.0, 0.0, 1.0);
            look_vector = rotate3d_x(look_vector, viewer_look_at_direction.y);
            look_vector = rotate3d_y(look_vector, viewer_look_at_direction.x);
            look_vector = (look_vector).normalize() * 2.0;

            let create_object_position = viewer_position + look_vector;
            self.object_creation_state.center = [create_object_position[0], create_object_position[1], create_object_position[2]];
            self.object_creation_state.color = self.selected_color;

            let res = world.insert_object(
                Vector3::from_vec(create_object_position.as_slice().into_iter().map(|x| *x as i32).collect()), 
                self.object_creation_state.clone()
            );

            if res.is_err() {
                console::log_1(&"No space left for creating new items!".into());
            }

            // just reset the selected color at the end
            let colors = generate_colors_list();
            let number_of_existing_objects = world.objects_associations.keys().len();
            let selected_color = Color32::from_rgb(
                colors[number_of_existing_objects % 12][0],
                colors[number_of_existing_objects % 12][1],
                colors[number_of_existing_objects % 12][2],
            );

            self.selected_color = selected_color;
        }
    }
}
