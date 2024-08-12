#![allow(clippy::undocumented_unsafe_blocks)]

use std::{cmp::min, f32::consts::PI, fmt::format, sync::Arc};

use eframe::{egui_glow, glow::HasContext};
use egui::{mutex::Mutex, vec2, Rect, Vec2};
use egui_glow::glow;
use math_vector::Vector;
use web_sys::{console, js_sys::Math};
use ::slice_of_array::prelude::*;

use crate::{camera::{self, Camera}, gpu_hash::{self, GPUHashTable}};

#[derive(Debug)]
struct Triangle {
    p0: Vector<f32>,
    p1: Vector<f32>,
    p2: Vector<f32>
}

#[derive(Copy, Clone)]
enum Max {
    Steps(usize),
    Distance(f64),
}

fn to_f64_slice(a: Vector<f32>) -> [f64; 3] {
    return [a.x as f64, a.y as f64, a.z as f64];
}

// Thanks to https://github.com/leroycep/ascii-raycaster/blob/master/src/main.rs
fn raymarch(pos: [f64; 3], dir: [f64; 3], end_pos: [f64; 3], max: Max) -> Vec<Vector<i32>> {
    let mut tiles_found: Vec<Vector<i32>> = vec![];

    let (max_steps, max_distance) = match max {
        Max::Steps(num) => (num, ::std::f64::INFINITY),
        Max::Distance(dist) => (::std::usize::MAX, dist),
    };
    let mut map_pos = [pos[0].round(), pos[1].round(), pos[2].round()];
    let dir2 = [dir[0]*dir[0], dir[1]*dir[1], dir[2]*dir[2]];
    let delta_dist = [(1.0             + dir2[1]/dir2[0] + dir2[2]/dir2[0]).sqrt(),
                      (dir2[0]/dir2[1] + 1.0             + dir2[2]/dir2[1]).sqrt(),
                      (dir2[0]/dir2[2] + dir2[1]/dir2[2] + 1.0            ).sqrt(),
    ];
    console::log_1(&format!("{:?}", delta_dist).into());
    let mut step = [0.0, 0.0, 0.0];
    let mut side_dist = [0.0, 0.0, 0.0];
    let mut side;
    for i in 0..3 {
        if dir[i] < 0.0 {
            step[i] = -1.0;
            side_dist[i] = (pos[i] - map_pos[i]) * delta_dist[i];
        } else {
            step[i] = 1.0;
            side_dist[i] = (map_pos[i] + 1.0 - pos[i]) * delta_dist[i];
        }
    }

    let mut last_distance = (Vector::new(map_pos[0], map_pos[1], map_pos[2]) - Vector::new(end_pos[0], end_pos[1], end_pos[2])).length();

    for _ in 0..max_steps {
        if side_dist[0] < side_dist[1] && side_dist[0] < side_dist[2] {
            side_dist[0] += delta_dist[0];
            map_pos[0] += step[0];
            side = 1;
        } else if side_dist[1] < side_dist[2] {
            side_dist[1] += delta_dist[1];
            map_pos[1] += step[1];
            side = 3;
        } else {
            side_dist[2] += delta_dist[2];
            map_pos[2] += step[2];
            side = 2;
        }
        let mut tile = 0;
        tiles_found.push(Vector::new(map_pos[0] as i32, map_pos[1] as i32, map_pos[2] as i32));

        if (Vector::new(map_pos[0], map_pos[1], map_pos[2]) - Vector::new(end_pos[0], end_pos[1], end_pos[2])).length() > last_distance { // check that we are getting closer
            console::log_1(&"exited ray caster when ray passed target".into());
            return tiles_found;
        }

        last_distance = (Vector::new(map_pos[0], map_pos[1], map_pos[2]) - Vector::new(end_pos[0], end_pos[1], end_pos[2])).length();

        if map_pos[0] as i32 == end_pos[0] as i32 && map_pos[1] as i32 == end_pos[1] as i32 && map_pos[2] as i32 == end_pos[2] as i32 {
            console::log_1(&"exited ray caster normally".into());
            return tiles_found;
        }
    }
    return tiles_found;
}

pub struct MainApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<MainGlowProgram>>,
    camera: Camera
}

impl MainApp {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            rotating_triangle: Arc::new(Mutex::new(MainGlowProgram::new(gl)?)),
            camera: Camera::new()
        })
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                    });

                    egui::Window::new("My Window").show(ctx, |ui| {
                       ui.label("Hello World!");
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

                        self.custom_painting(ui);
                    });
                });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
}

impl MainApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

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
    // faces_buffer: glow::Buffer
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

    fn paint(&self, gl: &glow::Context, camera: Camera, window_rect: Rect, grid: &GPUHashTable) {
        use glow::HasContext as _;

        let resolution_multiplier = 0.5;

        unsafe {
            gl.use_program(Some(self.main_image_program));

            let texture_resolution = [(window_rect.width() * resolution_multiplier) as i32, (window_rect.height() * resolution_multiplier) as i32];

            let framebuffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

            let texture_color_buffer = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_color_buffer));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGB as i32, texture_resolution[0], texture_resolution[1], 0, glow::RGB, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(texture_color_buffer), 0);

            let rbo = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, texture_resolution[0], texture_resolution[1]);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_STENCIL_ATTACHMENT, glow::RENDERBUFFER, Some(rbo));

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            // now this should happen every frame
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.enable(glow::DEPTH_TEST);

            gl.uniform_2_f32(
                gl.get_uniform_location(self.main_image_program, "u_rotation").as_ref(),
                camera.look_direction.x,
                camera.look_direction.y
            );

            console::log_1(&format!("camera position {:?}", camera.position).into());
            console::log_1(&format!("camera rotation {:?}", camera.look_direction).into());

            gl.uniform_3_f32(
                gl.get_uniform_location(self.main_image_program, "position").as_ref(),
                camera.position.x, 
                camera.position.y,
                camera.position.z
            );

            let mut list = [0u32; 3000];
            grid.opengl_compatible_objects_list(&mut list);

            console::log_1(&format!("{:?}", list).into());
            console::log_1(&format!("{:?}", grid.buckets).into());

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "buckets").as_ref(),
                &grid.buckets.as_slice()
            );

            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.main_image_program, "objects").as_ref(),
                &list
            );

            // gl.uniform_1_i32_slice(
            //     gl.get_uniform_location(self.main_image_program, "grid").as_ref(),
            //     vec![grid].flat().flat().flat() // 3 flats for 3 dimensions, sure
            // );

            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(self.main_image_program));
            gl.bind_vertex_array(Some(self.vertex_array));

            gl.active_texture(glow::TEXTURE0);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

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

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_color_buffer));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            // probably not the most efficient but oh well
            gl.delete_texture(texture_color_buffer);
            gl.delete_renderbuffer(rbo);
            gl.delete_framebuffer(framebuffer)
        }
    }
}
