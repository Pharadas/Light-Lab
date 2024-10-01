#![allow(clippy::undocumented_unsafe_blocks)]

use std::sync::Arc;

use eframe::egui_glow;
use egui::{mutex::Mutex, Button, Color32, ColorImage, ImageData, Rect, TextureOptions};
use egui_glow::glow;
use image::RgbaImage;
use nalgebra::{Complex, Matrix2, Vector2, Vector3};
use web_sys::console;

use crate::{camera::Camera, menus::MenusState, world::{LightPolarizationType, ObjectType, World, WorldObject}};

pub struct MainApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    glow_program: Arc<Mutex<MainGlowProgram>>,
    world: World,
    camera: Camera,
    time: f64,
    menus: MenusState,
}

impl MainApp {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;

        // This is a terrible way of doing this but no image loader will interpret a single format so i
        // guess this is what we are doing
        let image_sizes: Vec<[usize; 2]> = vec![
            // polarizers
            [1000, 711],
            [1000, 711],
            [1000, 408],
            [1000, 203],
            [1000, 485],
            [1000, 485],

            // phase retarders
            [1000, 428],
            [1000, 433],
            [1000, 142],
            [1000, 290],
            [1000, 150],
            [1000, 146],
            [1000, 126],
        ];

        let all_images = vec![
            RgbaImage::from_raw(image_sizes[0][0] as u32, image_sizes[0][1] as u32, include_bytes!("../assets/optical_objects_bytes/linear_horizontal_polarizer.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[1][0] as u32, image_sizes[1][1] as u32, include_bytes!("../assets/optical_objects_bytes/linear_vertical_polarizer.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[2][0] as u32, image_sizes[2][1] as u32, include_bytes!("../assets/optical_objects_bytes/linear_45_polarizer.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[3][0] as u32, image_sizes[3][1] as u32, include_bytes!("../assets/optical_objects_bytes/linear_theta_polarizer.bytes").to_vec()).unwrap(),

            RgbaImage::from_raw(image_sizes[4][0] as u32, image_sizes[4][1] as u32, include_bytes!("../assets/optical_objects_bytes/right_circular_polarizer.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[5][0] as u32, image_sizes[5][1] as u32, include_bytes!("../assets/optical_objects_bytes/left_circular_polarizer.bytes").to_vec()).unwrap(),

            RgbaImage::from_raw(image_sizes[6][0] as u32, image_sizes[6][1] as u32, include_bytes!("../assets/optical_objects_bytes/quarter_wave_vertical.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[7][0] as u32, image_sizes[7][1] as u32, include_bytes!("../assets/optical_objects_bytes/quarter_wave_horizontal.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[8][0] as u32, image_sizes[8][1] as u32, include_bytes!("../assets/optical_objects_bytes/quarter_wave_theta.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[9][0] as u32, image_sizes[9][1] as u32, include_bytes!("../assets/optical_objects_bytes/half_wave_theta.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[10][0] as u32, image_sizes[10][1] as u32, include_bytes!("../assets/optical_objects_bytes/half_wave_fast_horizontal_theta.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[11][0] as u32, image_sizes[11][1] as u32, include_bytes!("../assets/optical_objects_bytes/linear_phase_retarder_theta.bytes").to_vec()).unwrap(),
            RgbaImage::from_raw(image_sizes[12][0] as u32, image_sizes[12][1] as u32, include_bytes!("../assets/optical_objects_bytes/elliptical_phase_retarder.bytes").to_vec()).unwrap(),
        ];

        let screen_texture = cc.egui_ctx.load_texture(
            "screen",
            ImageData::Color(Arc::new(ColorImage::new([320, 80], Color32::TRANSPARENT))),
            TextureOptions::default(),
        );

        let debug_texture = cc.egui_ctx.load_texture(
            "screen",
            ImageData::Color(Arc::new(ColorImage::new([320, 80], Color32::TRANSPARENT))),
            TextureOptions::default(),
        );

        // load demo
        let mut demo_world = World::new();
        let demo_red_light = WorldObject { object_type: ObjectType::LightSource, rotation: [3.1415927, 1.5707964], center: [10.257881, 2.1159875, 11.990719], color: Color32::from_rgb(255, 1, 1), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal };
        let demo_blue_light = WorldObject { object_type: ObjectType::LightSource, rotation: [3.1415927, 1.5707964], center: [11.257681, 2.1159875, 12.010717], color: Color32::from_rgb(1, 1, 255), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal };
        let demo_wall = WorldObject { object_type: ObjectType::RoundWall, rotation: [3.1415927, 0.85794735], center: [10.26795, 3.0669506, 16.072115], color: Color32::from_rgb(21, 122, 189), width: 0.5, height: 0.5, radius: 0.5, polarization: Vector2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::NotPolarized };

        demo_world.insert_object(Vector3::from_vec(demo_red_light.center.into_iter().map(|x| x as i32).collect()), demo_red_light);
        demo_world.insert_object(Vector3::from_vec(demo_blue_light.center.into_iter().map(|x| x as i32).collect()), demo_blue_light);
        demo_world.insert_object(Vector3::from_vec(demo_wall.center.into_iter().map(|x| x as i32).collect()), demo_wall);

        Some(Self {
            glow_program: Arc::new(Mutex::new(MainGlowProgram::new(gl)?)),
            world: demo_world,
            camera: Camera::new(),
            time: 0.0,
            menus: MenusState::new(screen_texture, debug_texture, all_images, image_sizes)
        })
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.request_repaint_after_secs(0.033);
            egui::ScrollArea::both()
                .auto_shrink(false)
                .show(ui, |ui| {
                    egui::Window::new("Main menu").show(ctx, |ui| {
                        ui.label(format!("Current position: {:?}, {:?}, {:?}", self.camera.position.x.round(), self.camera.position.y.round(), self.camera.position.z.round()));
                        ui.add(egui::Slider::new(&mut self.glow_program.lock().desired_scaling_factor, 0.1..=1.0).text("Scaling factor"));

                        ui.add(egui::Slider::new(&mut self.glow_program.lock().cube_scaling_factor, 0.05..=3.0).text("Cube size in meters"));

                        if self.menus.should_display_debug_menu {
                            if ui.add(Button::new("Hide debug menu")).clicked() {
                                self.menus.should_display_debug_menu = false;
                            }
                        } else {
                            if ui.add(Button::new("Display debug menu")).clicked() {
                                self.menus.should_display_debug_menu = true;
                            }
                        }
                    });

                    if self.glow_program.lock().currently_selected_object != 0 {
                        egui::Window::new("Object inspector").show(ctx, |ui| {
                            self.menus.inspect_object_menu(ui, &mut self.world, self.time, &mut self.glow_program.lock().currently_selected_object);
                        });
                    }

                    egui::Window::new("Object creator").show(ctx, |ui| {
                        self.menus.object_creation_menu(ui, &mut self.world, self.camera.position, Vector2::new(self.camera.look_direction.x, self.camera.look_direction.y));
                    });

                    if self.menus.should_display_debug_menu {
                        egui::Window::new("Debug menu").max_height(500.0).show(ctx, |ui| {
                            self.menus.debug_menu(ui, &mut self.world, self.glow_program.lock().clone());
                        });

                    }

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        if ui.input(|i| i.key_pressed(egui::Key::A)) {
                            self.camera.update(egui::Key::A);
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::D)) {
                            self.camera.update(egui::Key::D);
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::S)) {
                            self.camera.update(egui::Key::S);
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::W)) {
                            self.camera.update(egui::Key::W);
                        }
                    });

                    self.custom_painting(ui);
                });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
if let Some(gl) = gl {
            self.glow_program.lock().destroy(gl);
        }
    }
}

impl MainApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        let curr_response = ui.interact(ui.min_rect(), egui::Id::new("Some Id"), egui::Sense::click());
        let current_texture_resolution = self.glow_program.lock().current_texture_resolution.clone();
        let objects_found = self.glow_program.lock().objects_found.clone();

        if curr_response.clicked() {
            console::log_1(&format!("original hover position: {:?}", curr_response.hover_pos()).into());

            let texture_coordinates_hover_pos = [
                ((curr_response.hover_pos().unwrap().x * current_texture_resolution[0] as f32) / rect.right_bottom().x) as i32,
                ((curr_response.hover_pos().unwrap().y * current_texture_resolution[1] as f32) / rect.right_bottom().y) as i32
            ];

            console::log_1(&format!("texture coordinates hover position: {:?}", texture_coordinates_hover_pos).into());

            let object_found_index = objects_found[((((current_texture_resolution[1] - texture_coordinates_hover_pos[1]) * current_texture_resolution[0]) + texture_coordinates_hover_pos[0]) * 4) as usize];
            self.glow_program.lock().currently_selected_object = object_found_index as usize;

            // console::log_1(&format!("value at texture space coordinates: {:?}", objects_found.len()).into());
            console::log_1(&format!("value at texture space coordinates: {:?}", objects_found[((((current_texture_resolution[1] - texture_coordinates_hover_pos[1]) * current_texture_resolution[0]) + texture_coordinates_hover_pos[0]) * 4) as usize]).into());
        }

        if response.clicked_elsewhere() {
        //     console::log_1(&format!("main window clicked!").into());
            console::log_1(&format!("{:?}", response.hover_pos()).into());
        }

        self.time += 0.01;

        self.camera.look_direction += response.drag_motion() * 0.01;
        self.camera.look_direction.y = self.camera.look_direction.y.clamp(-1.4, 1.4);

        // Clone locals so we can move them into the paint callback:
        let rotating_triangle = self.glow_program.clone();
        let sent_camera = self.camera.clone();
        let world = self.world.clone();
        let time = self.time.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), sent_camera, rect, &world, time as f32);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

#[derive(Clone)]
pub struct MainGlowProgram {
    pub main_image_program: glow::Program,
    pub present_program: glow::Program,
    pub vertex_array: glow::VertexArray,
    pub current_texture_resolution: [i32; 2],
    pub objects_found: Vec<u8>,
    pub desired_scaling_factor: f32,
    pub cube_scaling_factor: f32,
    pub currently_selected_object: usize
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl MainGlowProgram {
    fn new(gl: &glow::Context) -> Option<Self> {
        use glow::HasContext as _;

        let shader_version = egui_glow::ShaderVersion::get(gl);

        unsafe {
            // create the program to render the ray marched image to a texture,
            // this is done simply so that users have control over the resolution
            // of the resulting image
            // the program to simply render the texture on the screen
            let offscreen_program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                log::warn!(
                    "Custom 3D painting hasn't been ported to {:?}",
                    shader_version
                );
                return None;
            }

            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("./gpu-code/main.vert"),
                include_str!("./gpu-code/main.frag")
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!(
                            "{}\n{}",
                            shader_version.version_declaration(),
                            shader_source
                        ),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(offscreen_program, shader);
                    shader
                })
                .collect();

            gl.link_program(offscreen_program);
            assert!(
                gl.get_program_link_status(offscreen_program),
                "{}",
                gl.get_program_info_log(offscreen_program)
            );

            for shader in shaders {
                gl.detach_shader(offscreen_program, shader);
                gl.delete_shader(shader);
            }

            // now we create a program which will simply take in a texture, sample it
            // and present it to the screen on any resolution
            let present_to_screen_program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                log::warn!(
                    "Custom 3D painting hasn't been ported to {:?}",
                    shader_version
                );
                return None;
            }

            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("./gpu-code/present.vert"),
                include_str!("./gpu-code/present.frag")
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!(
                            "{}\n{}",
                            shader_version.version_declaration(),
                            shader_source
                        ),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(present_to_screen_program, shader);
                    shader
                })
                .collect();

            gl.link_program(present_to_screen_program);
            assert!(
                gl.get_program_link_status(present_to_screen_program),
                "{}",
                gl.get_program_info_log(present_to_screen_program)
            );

            for shader in shaders {
                gl.detach_shader(present_to_screen_program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Some(Self {
                main_image_program: offscreen_program,
                present_program: present_to_screen_program,
                vertex_array,
                current_texture_resolution: [0, 0],
                objects_found: vec![0u8],
                desired_scaling_factor: 0.25,
                cube_scaling_factor: 3.0,
                currently_selected_object: 0
            })
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.main_image_program);
            gl.delete_program(self.present_program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&mut self, gl: &glow::Context, camera: Camera, window_rect: Rect, world: &World, time: f32) {
        use glow::HasContext as _;

        let resolution_multiplier = self.desired_scaling_factor;

        unsafe {
            gl.use_program(Some(self.main_image_program));

            let texture_resolution = [(window_rect.width() * resolution_multiplier) as i32, (window_rect.height() * resolution_multiplier) as i32];

            self.current_texture_resolution = texture_resolution;

            let framebuffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

            let color_buffer = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGB as i32, texture_resolution[0], texture_resolution[1], 0, glow::RGB, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(color_buffer), 0);

            let object_found = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(object_found));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGB as i32, texture_resolution[0], texture_resolution[1], 0, glow::RGB, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT1, glow::TEXTURE_2D, Some(object_found), 0);

            let rbo = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, texture_resolution[0], texture_resolution[1]);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_STENCIL_ATTACHMENT, glow::RENDERBUFFER, Some(rbo));

            // i would have made my rust structs gpu compatible with some crate
            // but for some fucking reason glow doesn't offer any 
            // uniform_1_u8 function
            gl.uniform_2_f32(
                gl.get_uniform_location(self.main_image_program, "u_rotation").as_ref(),
                camera.look_direction.x,
                camera.look_direction.y
            );

            gl.uniform_3_f32(
                gl.get_uniform_location(self.main_image_program, "position").as_ref(),
                camera.position.x, 
                camera.position.y,
                camera.position.z
            );

            let mut list = [0u32; 3000];
            world.hash_map.opengl_compatible_objects_list(&mut list);

            gl.uniform_1_f32(
                gl.get_uniform_location(self.main_image_program, "time").as_ref(),
                time
            );

            gl.uniform_1_f32(
                gl.get_uniform_location(self.main_image_program, "cube_scaling_factor").as_ref(),
                self.cube_scaling_factor
            );

            gl.uniform_1_u32(
                gl.get_uniform_location(self.main_image_program, "light_sources_count").as_ref(),
                world.light_sources.len() as u32
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "lights_definitions_indices").as_ref(),
                &world.light_sources.as_slice()
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "buckets").as_ref(),
                &world.hash_map.buckets.as_slice()
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "objects").as_ref(),
                &list
            );

            gl.uniform_2_f32(
                gl.get_uniform_location(self.main_image_program, "viewport_dimensions").as_ref(),
                texture_resolution[0] as f32,
                texture_resolution[0] as f32
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "objects_definitions").as_ref(),
                world.get_gpu_compatible_world_objects_list().as_slice()
            );

            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(self.main_image_program));
            gl.bind_vertex_array(Some(self.vertex_array));

            gl.active_texture(glow::TEXTURE0);
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            gl.bind_texture(glow::TEXTURE_2D, Some(object_found));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT1, glow::TEXTURE_2D, Some(object_found), 0);
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);

            // read the texture contents
            let mut buffer = vec![0u8; (texture_resolution[0] * texture_resolution[1] * 4) as usize];

            gl.read_buffer(glow::COLOR_ATTACHMENT1);
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE {
                gl.read_pixels(
                    0, 
                    0, 
                    texture_resolution[0],
                    texture_resolution[1],
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelPackData::Slice(&mut buffer)
                );
                // console::log_1(&format!("{:?}", buffer).into());
            } else {
                console::log_1(&format!("couldnt read framebuffer as it wasnt done").into());
            }

            self.objects_found = buffer;

            // present to screen
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.disable(glow::DEPTH_TEST);

            gl.use_program(Some(self.present_program));

            gl.bind_vertex_array(Some(self.vertex_array));

            gl.uniform_1_i32(
                gl.get_uniform_location(self.present_program, "screenTexture").as_ref(),
                0
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.present_program, "objects_found").as_ref(),
                1
            );

            gl.uniform_2_f32(
                gl.get_uniform_location(self.present_program, "viewport_dimensions").as_ref(),
                window_rect.width(),
                window_rect.height()
            );

            gl.uniform_1_u32(
                gl.get_uniform_location(self.present_program, "selected_object").as_ref(),
                self.currently_selected_object as u32
            );

            gl.uniform_1_f32(
                gl.get_uniform_location(self.present_program, "time").as_ref(),
                time
            );

            // console::log_1(&format!("{:?}", texture_resolution).into());

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));

            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(object_found));

            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            // probably not the most efficient but oh well
            gl.delete_texture(color_buffer);
            gl.delete_texture(object_found);
            gl.delete_renderbuffer(rbo);
            gl.delete_framebuffer(framebuffer)
        }
    }
}
