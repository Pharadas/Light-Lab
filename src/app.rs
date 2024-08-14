#![allow(clippy::undocumented_unsafe_blocks)]

use std::sync::Arc;

use as_bytes::AsBytes;
use eframe::{egui_glow, glow::HasContext};
use egui::{color_picker::{color_picker_color32, Alpha}, epaint::color, mutex::Mutex, Color32, Rect, RichText, Widget, WidgetText};
use egui_glow::glow;
use web_sys::console;

use crate::{camera::Camera, menus::{MenusState, OpticalObject}, rasterizer::World};

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
        Some(Self {
            glow_program: Arc::new(Mutex::new(MainGlowProgram::new(gl)?)),
            camera: Camera::new(),
            world: World::new(),
            time: 0.0,
            menus: MenusState::new()
        })
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink(false)
                .show(ui, |ui| {
                    egui::Window::new("Main menu").show(ctx, |ui| {
                        ui.label(format!("Current position: {:?}, {:?}, {:?}", self.camera.position.x.round(), self.camera.position.y.round(), self.camera.position.z.round()));
                    });

                    egui::Window::new("Object creator").show(ctx, |ui| {
                        self.menus.select_object_menu(ui, &mut self.world, &self.camera.position);
                        // color_picker_color32(ui, &mut Color32::from_rgb(255, 20, 20), Alpha::Opaque);
                    });

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        if ui.input(|i| i.key_pressed(egui::Key::A)) {
                            self.camera.update(egui::Key::A);
                            console::log_1(&"pressed A".into());
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::D)) {
                            self.camera.update(egui::Key::D);
                            console::log_1(&"pressed D".into());
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::S)) {
                            self.camera.update(egui::Key::S);
                            console::log_1(&"pressed S".into());
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::W)) {
                            self.camera.update(egui::Key::W);
                            console::log_1(&"pressed W".into());
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

        // console::log_1(response.clicked().)

        self.camera.look_direction += response.drag_motion() * 0.01;
        self.camera.look_direction.y = self.camera.look_direction.y.clamp(-1.4, 1.4);

        // Clone locals so we can move them into the paint callback:
        let rotating_triangle = self.glow_program.clone();
        let sent_camera = self.camera.clone();
        let world = self.world.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), sent_camera, rect, &world);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

struct MainGlowProgram {
    main_image_program: glow::Program,
    present_program: glow::Program,
    vertex_array: glow::VertexArray,
    current_texture_resolution: [i32; 2],
    objects_found: Vec<u8>
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
                include_str!("./main.vert"),
                include_str!("./main.frag")
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
                include_str!("./present.vert"),
                include_str!("./present.frag")
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
                objects_found: vec![0u8]
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

    fn paint(&mut self, gl: &glow::Context, camera: Camera, window_rect: Rect, world: &World) {
        use glow::HasContext as _;

        let resolution_multiplier = 0.5;

        unsafe {
            gl.use_program(Some(self.main_image_program));

            // let objects_buffer = gl.create_buffer().unwrap();
            // gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(objects_buffer));
            // gl.buffer_data_u8_slice(glow::SHADER_STORAGE_BUFFER, [1u32].as_bytes(), glow::DYNAMIC_READ);
            // gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 2, Some(objects_buffer));

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

            gl.uniform_2_f32(
                gl.get_uniform_location(self.main_image_program, "u_rotation").as_ref(),
                camera.look_direction.x,
                camera.look_direction.y
            );

            // console::log_1(&format!("camera position {:?}", camera.position).into());
            // console::log_1(&format!("camera rotation {:?}", camera.look_direction).into());

            gl.uniform_3_f32(
                gl.get_uniform_location(self.main_image_program, "position").as_ref(),
                camera.position.x, 
                camera.position.y,
                camera.position.z
            );

            let mut list = [0u32; 3000];
            world.hash_map.opengl_compatible_objects_list(&mut list);

            // console::log_1(&format!("{:?}", list).into());
            // console::log_1(&format!("{:?}", world.hash_map.buckets).into());

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "buckets").as_ref(),
                &world.hash_map.buckets.as_slice()
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "objects").as_ref(),
                &list
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

            gl.uniform_2_f32(
                gl.get_uniform_location(self.present_program, "viewport_dimensions").as_ref(),
                window_rect.width(),
                window_rect.height()
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            // probably not the most efficient but oh well
            gl.delete_texture(color_buffer);
            gl.delete_texture(object_found);
            gl.delete_renderbuffer(rbo);
            gl.delete_framebuffer(framebuffer)
        }
    }
}
