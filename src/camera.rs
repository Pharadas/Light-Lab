// dont like mixing them but i kinda want to use dot products n stuff
use egui::Vec2;
use nalgebra::Vector3;

#[derive(Clone, Copy)]
pub struct Camera {
    pub look_direction: Vec2,
    pub position: Vector3<f32>,
}

// maybe should move this to a math.rs module or something
pub fn rotate3d_y(v: Vector3<f32>, a: f32) -> Vector3<f32> {
    let cos_a = a.cos();
    let sin_a = a.sin();

    return Vector3::new(
        v.x * cos_a + v.z * sin_a,
        v.y,
        -v.x * sin_a + v.z * cos_a
    );
}

pub fn rotate3d_x(v: Vector3<f32>, a: f32) -> Vector3<f32> {
    let cos_a = a.cos();
    let sin_a = a.sin();

    return Vector3::new(
        v.x,
        v.y * cos_a - v.z * sin_a,
        v.y * sin_a + v.z * cos_a
    );
}

impl Camera {
    pub fn new() -> Camera {
        return Camera {
            look_direction: Vec2::new(0.0, 0.0),
            position: Vector3::new(10., 10., 10.),
        }
    }

    pub fn update(&mut self, key: egui::Key) {
        let mut movement: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        match key {
            egui::Key::D => movement.x += 1.0,
            egui::Key::A => movement.x -= 1.0,
            egui::Key::W => movement.z += 1.0,
            egui::Key::S => movement.z -= 1.0,

            _ => {}
        }

        movement = rotate3d_x(movement, self.look_direction.y);
        movement = rotate3d_y(movement, self.look_direction.x);
        movement = movement.normalize();

        // self.rotate3d_x(&mut movement);
        // self.rotate3d_y(&mut movement);
        movement = movement.normalize() * 0.5;

        self.position += movement;
        self.position.x = self.position.x.clamp(2.0, 20.);
        self.position.y = self.position.y.clamp(2.0, 20.);
        self.position.z = self.position.z.clamp(2.0, 20.);
    }
}
