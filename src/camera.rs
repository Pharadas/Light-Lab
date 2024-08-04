// dont like mixing them but i kinda want to use dot products n stuff
use egui::Vec2;
use math_vector::Vector;

#[derive(Clone, Copy)]
pub struct Camera {
    pub look_direction: Vec2,
    pub position: Vector<f32>,
    up: Vector<f32>
}

impl Camera {
    pub fn new() -> Camera {
        return Camera {
            look_direction: Vec2::new(0.0, 0.0),
            position: Vector::new(0.0, 0.0, 0.0),
            up: Vector::new(0.0, 1.0, 0.0)
        }
    }

    fn rotate3d_y(&self, v: Vector<f32>, a: f32) -> Vector<f32> {
        let cos_a = a.cos();
        let sin_a = a.sin();

        return Vector::new(
            v.x * cos_a + v.z * sin_a,
            v.y,
            -v.x * sin_a + v.z * cos_a
        );
    }

    fn rotate3d_x(&self, v: Vector<f32>, a: f32) -> Vector<f32> {
        let cos_a = a.cos();
        let sin_a = a.sin();

        return Vector::new(
            v.x,
            v.y * cos_a - v.z * sin_a,
            v.y * sin_a + v.z * cos_a
        );
    }

    pub fn update(&mut self, key: egui::Key) {
        let mut movement: Vector<f32> = Vector::new(0.0, 0.0, 0.0);

        match key {
            egui::Key::D => movement.x += 1.0,
            egui::Key::A => movement.x -= 1.0,
            egui::Key::W => movement.z += 1.0,
            egui::Key::S => movement.z -= 1.0,

            _ => {}
        }

        movement = self.rotate3d_x(movement, self.look_direction.y);
        movement = self.rotate3d_y(movement, self.look_direction.x);
        movement = movement.normalize();

        // self.rotate3d_x(&mut movement);
        // self.rotate3d_y(&mut movement);
        movement = movement.normalize();

        self.position += movement;
    }
}
