#![allow(clippy::undocumented_unsafe_blocks)]

use std::{cmp::min, fmt::format, sync::Arc};

use eframe::{egui_glow, glow::HasContext};
use egui::{mutex::Mutex, vec2, Rect, Vec2};
use egui_glow::glow;
use math_vector::Vector;
use web_sys::console;
use ::slice_of_array::prelude::*;

struct Triangle {
    p0: Vector<f32>,
    p1: Vector<f32>,
    p2: Vector<f32>
}

fn sign(v: Vector<f32>) -> Vector<i32> {
    let a: i32 = (v.x > 0.0) as i32 - (v.x < 0.0) as i32;
    let b: i32 = (v.y > 0.0) as i32 - (v.y < 0.0) as i32;
    let c: i32 = (v.z > 0.0) as i32 - (v.z < 0.0) as i32;

    return Vector::new(a, b, c)
}

fn step_ray(mask: &mut Vector<bool>, side_dist: &mut Vector<f32>, delta_dist: Vector<f32>, map_pos: &mut Vector<i32>, step: Vector<i32>) {
    *mask = Vector::new(
        side_dist.x <= side_dist.y.min(side_dist.z),
        side_dist.y <= side_dist.z.min(side_dist.x),
        side_dist.z <= side_dist.x.min(side_dist.y)
    );

    let i32_mask = Vector::new(mask.x as i32, mask.y as i32, mask.z as i32);

    *side_dist += i32_mask.as_f32s().mul_components(delta_dist);
    *map_pos += i32_mask.mul_components(step);
}

fn iterate_ray(initial_position: Vector<f32>, end_position: Vector<f32>) -> Vec<Vector<i32>> {
    // setup ray
    // this will be way too verbose
    let direction = (end_position - initial_position).normalize();
    let mut map_pos:    Vector<i32> = Vector::new(initial_position.x as i32, initial_position.y as i32, initial_position.z as i32);
    let delta_dist: Vector<f32> = Vector::new(1.0f32 / direction.x.abs(), 1.0f32 / direction.y.abs(), 1.0f32 / direction.z.abs());
    let step:       Vector<i32> = sign(direction);
    let mut side_dist:  Vector<f32> = sign(direction).as_f32s().mul_components(map_pos.as_f32s() - initial_position) + ((sign(direction).as_f32s() * 0.5) + Vector::new(0.5, 0.5, 0.5)).mul_components(delta_dist);
    let mut mask:       Vector<bool> = Vector::new(
        side_dist.x <= side_dist.y.min(side_dist.z),
        side_dist.y <= side_dist.z.min(side_dist.x),
        side_dist.z <= side_dist.x.min(side_dist.y)
    );

    let mut positions_found: Vec<Vector<i32>> = vec![];

    while map_pos != end_position.as_i32s() {
        positions_found.push(map_pos.clone()); // not should if i have to clone it but i'd rather be explicit
        step_ray(&mut mask, &mut side_dist, delta_dist, &mut map_pos, step);
    }

    return positions_found;
}

pub struct MainApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<MainGlowProgram>>,
    rotation: Vec2
}

impl MainApp {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            rotating_triangle: Arc::new(Mutex::new(MainGlowProgram::new(gl)?)),
            rotation: vec2(0., 0.)
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

        self.rotation += response.drag_motion() * 0.01;

        // Clone locals so we can move them into the paint callback:
        let rotation = self.rotation;
        let rotating_triangle = self.rotating_triangle.clone();

        // create grid to send to gpu
        let sample_triangle = Triangle {
            p0: Vector::new(5.06325, 5.0359793, 5.0420873),
            p1: Vector::new(-5.06275, 5.0360343, 5.0425949),
            p2: Vector::new(-5.0645, 5.0365101, 5.0404362),
        };

        let mut cubic_grid = [[[0i32; 11]; 11]; 11];

        let a_through_b_rasterized = iterate_ray(sample_triangle.p0, sample_triangle.p1);

        for position in a_through_b_rasterized {
            cubic_grid[(position.x + 5) as usize][(position.y + 5) as usize][(position.z + 5) as usize] = 1;
            // now just keep firing rays to every position and rasterizing
            let c_through_position_rasterized = iterate_ray(sample_triangle.p2, position.as_f32s() + Vector::new(0.5, 0.5, 0.5));
            // just put it into the grid
            for new_position in c_through_position_rasterized {
                cubic_grid[(new_position.x + 5) as usize][(new_position.y + 5) as usize][(new_position.z + 5) as usize] = 1;
            }
        }

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), rotation, rect, cubic_grid);
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

    fn paint(&self, gl: &glow::Context, rotation: Vec2, window_rect: Rect, grid: [[[i32; 11]; 11]; 11]) {
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
                rotation.x,
                rotation.y
            );

            gl.uniform_1_i32_slice(
                gl.get_uniform_location(self.main_image_program, "grid").as_ref(),
                vec![grid].flat().flat().flat() // 3 flats for 3 dimensions, sure
            );

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
