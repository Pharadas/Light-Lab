use std::fmt::{self, Display, Formatter};
use egui::Color32;
use nalgebra::{Complex, Matrix2, Vector2, Vector3};

use crate::world::{self, LightPolarizationType, ObjectType, World, WorldObject};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Demo {
    None,
    LightProfile,
    SimpleInterferenceDemo,
    DoubleSlit,
    TripleSlit,
    UncoordinatedInterference,
    CoordinatedInterference
}

// Needed for the drop down list
impl Display for Demo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "No demo"),
            Self::LightProfile => write!(f, "Light profile demo"),
            Self::SimpleInterferenceDemo => write!(f, "Simple interference demo"),
            Self::DoubleSlit => write!(f, "Double slit demo"),
            Self::TripleSlit => write!(f, "Triple slit demo"),
            Self::UncoordinatedInterference => write!(f, "Uncoordinated interference demo"),
            Self::CoordinatedInterference => write!(f, "Coordinated interference demo"),
        }
    }
}

pub fn no_demo() -> World {
    return World::new()
}

pub fn light_profile() -> World {
    let mut demo_world = World::new();

    let mut light = WorldObject::new();

    light.object_type = ObjectType::LightSource;
    light.center = [2.0, 2.360543, 14.195536];
    light.rotation = [-1.57, 0.0];
    light.radius = 1.0;
    light.color = Color32::from_rgb(172, 0, 35);
    light.polarization_type = LightPolarizationType::LinearHorizontal;
    light.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(light.center.into_iter().map(|x| x as i32).collect()), light).unwrap();
    return demo_world;
}

pub fn simple_interference_demo() -> World {
    let mut demo_world = World::new();

    let mut demo_red_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(35, 1, 1), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::FRONT, aligned_distance: 0.5, object_aligned_to_self: 0, wavelength: 0.001 };
    let mut demo_blue_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(1, 1, 35), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::FRONT, aligned_distance: 0.0, object_aligned_to_self: 9, wavelength: 0.001 };
    demo_world.aligned_objects.insert(9);

    demo_red_light.set_light_polarization();
    demo_blue_light.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(demo_red_light.center.into_iter().map(|x| x as i32).collect()), demo_red_light).unwrap();
    demo_world.insert_object(Vector3::from_vec(demo_blue_light.center.into_iter().map(|x| x as i32).collect()), demo_blue_light).unwrap();

    return demo_world
}

pub fn double_slit_demo() -> World {
    let mut demo_world = World::new();

    let mut demo_red_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(35, 1, 1), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: 0.5, object_aligned_to_self: 0, wavelength: 0.05 };
    let mut demo_blue_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(1, 1, 35), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::FRONT, aligned_distance: 0.0, object_aligned_to_self: 9, wavelength: 0.05 };
    demo_world.aligned_objects.insert(9);

    demo_red_light.set_light_polarization();
    demo_blue_light.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(demo_red_light.center.into_iter().map(|x| x as i32).collect()), demo_red_light).unwrap();
    demo_world.insert_object(Vector3::from_vec(demo_blue_light.center.into_iter().map(|x| x as i32).collect()), demo_blue_light).unwrap();

    return demo_world
}

pub fn triple_slit_demo() -> World {
    let mut demo_world = World::new();

    let mut demo_red_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(35, 1, 1), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: 0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut demo_green_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(1, 35, 1), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::FRONT, aligned_distance: 0.0, object_aligned_to_self: 7, wavelength: 0.1 };
    let mut demo_blue_light = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [13.749462, 13.868861, 16.94075], color: Color32::from_rgb(1, 1, 35), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    demo_world.aligned_objects.insert(7);
    demo_world.aligned_objects.insert(9);

    demo_red_light.set_light_polarization();
    demo_green_light.set_light_polarization();
    demo_blue_light.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(demo_red_light.center.into_iter().map(|x| x as i32).collect()), demo_red_light).unwrap();
    demo_world.insert_object(Vector3::from_vec(demo_green_light.center.into_iter().map(|x| x as i32).collect()), demo_green_light).unwrap();
    demo_world.insert_object(Vector3::from_vec(demo_blue_light.center.into_iter().map(|x| x as i32).collect()), demo_blue_light).unwrap();

    return demo_world
}

pub fn uncoordinated_interference_demo() -> World {
    let mut demo_world = World::new();

    let mut l1 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [15.375362, 15.805714, 12.920403], color: Color32::from_rgb(164, 30, 150), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: 0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l2 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.55704, 15.805714, 12.89948], color: Color32::from_rgb(164, 30, 150), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::FRONT, aligned_distance: 0.0, object_aligned_to_self: 7, wavelength: 0.1 };
    let mut l3 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.62066, 15.051637, 12.97835], color: Color32::from_rgb(9, 62, 36), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l4 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.778408, 16.207035, 12.74316], color: Color32::from_rgb(200, 40, 15), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l5 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.81051, 15.668215, 12.135378], color: Color32::from_rgb(52, 112, 17), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.5, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l6 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [15.516808, 15.95551, 13.701332], color: Color32::from_rgb(78, 175, 51), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.5, object_aligned_to_self: 0, wavelength: 0.1 };

    l1.set_light_polarization();
    l2.set_light_polarization();
    l3.set_light_polarization();
    l4.set_light_polarization();
    l5.set_light_polarization();
    l6.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(l1.center.into_iter().map(|x| x as i32).collect()), l1).unwrap();
    demo_world.insert_object(Vector3::from_vec(l2.center.into_iter().map(|x| x as i32).collect()), l2).unwrap();
    demo_world.insert_object(Vector3::from_vec(l3.center.into_iter().map(|x| x as i32).collect()), l3).unwrap();
    demo_world.insert_object(Vector3::from_vec(l4.center.into_iter().map(|x| x as i32).collect()), l4).unwrap();
    demo_world.insert_object(Vector3::from_vec(l5.center.into_iter().map(|x| x as i32).collect()), l5).unwrap();
    demo_world.insert_object(Vector3::from_vec(l6.center.into_iter().map(|x| x as i32).collect()), l6).unwrap();

    return demo_world
}

pub fn coordinated_interference_demo() -> World {
    let mut demo_world = World::new();
    demo_world.aligned_objects.insert(2);
    demo_world.aligned_objects.insert(5);
    demo_world.aligned_objects.insert(8);
    demo_world.aligned_objects.insert(6);
    demo_world.aligned_objects.insert(7);
    demo_world.aligned_objects.insert(1);

    let mut l1 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [12.466017, 13.034395, 15.146756], color: Color32::from_rgb(52, 112, 17), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::RIGHT, aligned_distance: 0.0, object_aligned_to_self: 8, wavelength: 0.1 };

    let mut l2 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.55704, 15.805714, 12.89948], color: Color32::from_rgb(164, 30, 150), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 9, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 7, wavelength: 0.1 };
    let mut l3 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.62066, 15.051637, 12.97835], color: Color32::from_rgb(9, 62, 36), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 8, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l4 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.778408, 16.207035, 12.74316], color: Color32::from_rgb(200, 40, 15), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 5, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 0, wavelength: 0.1 };
    let mut l5 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [14.81051, 15.668215, 12.135378], color: Color32::from_rgb(52, 112, 17), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 4, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 6, wavelength: 0.1 };

    let mut l6 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [12.466017, 13.334396, 15.146756], color: Color32::from_rgb(78, 175, 51), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::RIGHT, aligned_distance: 0.0, object_aligned_to_self: 5, wavelength: 0.1 };

    let mut l7 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [12.466017, 13.6343975, 15.146756], color: Color32::from_rgb(78, 175, 51), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 0, alignment: world::Alignment::RIGHT, aligned_distance: 0.0, object_aligned_to_self: 2, wavelength: 0.1 };

    let mut l8 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [15.516808, 15.95551, 13.701332], color: Color32::from_rgb(78, 175, 51), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 3, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 1, wavelength: 0.1 };
    let mut l9 = WorldObject { object_type: ObjectType::LightSource, rotation: [0.0, 0.0], center: [15.516808, 15.95551, 13.701332], color: Color32::from_rgb(78, 175, 51), width: 0.5, height: 0.5, radius: 0.1, polarization: Vector2::new(Complex { re: 1.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), jones_matrix: Matrix2::new(Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: 0.0, im: 0.0 }), polarization_type: LightPolarizationType::LinearHorizontal, aligned_to_object: 2, alignment: world::Alignment::RIGHT, aligned_distance: -0.4, object_aligned_to_self: 0, wavelength: 0.1 };

    l1.set_light_polarization();
    l2.set_light_polarization();
    l3.set_light_polarization();
    l4.set_light_polarization();
    l5.set_light_polarization();
    l6.set_light_polarization();
    l7.set_light_polarization();
    l8.set_light_polarization();
    l9.set_light_polarization();

    demo_world.insert_object(Vector3::from_vec(l1.center.into_iter().map(|x| x as i32).collect()), l1).unwrap();
    demo_world.insert_object(Vector3::from_vec(l2.center.into_iter().map(|x| x as i32).collect()), l2).unwrap();
    demo_world.insert_object(Vector3::from_vec(l3.center.into_iter().map(|x| x as i32).collect()), l3).unwrap();
    demo_world.insert_object(Vector3::from_vec(l4.center.into_iter().map(|x| x as i32).collect()), l4).unwrap();
    demo_world.insert_object(Vector3::from_vec(l5.center.into_iter().map(|x| x as i32).collect()), l5).unwrap();
    demo_world.insert_object(Vector3::from_vec(l6.center.into_iter().map(|x| x as i32).collect()), l6).unwrap();
    demo_world.insert_object(Vector3::from_vec(l7.center.into_iter().map(|x| x as i32).collect()), l7).unwrap();
    demo_world.insert_object(Vector3::from_vec(l8.center.into_iter().map(|x| x as i32).collect()), l8).unwrap();
    demo_world.insert_object(Vector3::from_vec(l9.center.into_iter().map(|x| x as i32).collect()), l9).unwrap();

    return demo_world
}
