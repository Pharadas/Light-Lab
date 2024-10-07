use std::{collections::{HashMap, HashSet}, f32::consts::PI, fmt::{self, Display, Formatter}, u32};
use egui::Color32;
use nalgebra::{Complex, ComplexField, Matrix2, Vector2, Vector3};
use web_sys::console;
use serde::{Deserialize, Serialize};

use crate::{camera::{rotate3d_x, rotate3d_y}, gpu_hash::GPUHashTable, util::i32_to_u32_vec};

// WorldObject.type possible values
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    CubeWall = 0,                   // Filled cube that can only be in uvec3 positions
    SquareWall = 1,                 // Infinitesimally thin square wall
    RoundWall = 2,                  // Infinitesimally thin round wall
    LightSource = 3,                // Sphere that represents a light source
    OpticalObjectCube = 4,          // An object represented using a jones matrix
    OpticalObjectSquareWall = 5,    // An object represented using a jones matrix
    OpticalObjectRoundWall = 6,     // An object represented using a jones matrix
}

// Needed for the drop down list
impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CubeWall => write!(f, "Wall (cube)"),
            Self::SquareWall => write!(f, "Wall (square)"),
            Self::RoundWall => write!(f, "Wall (round)"),
            Self::LightSource => write!(f, "Light source (sphere)"),
            Self::OpticalObjectCube => write!(f, "Optical object (cube)"),
            Self::OpticalObjectSquareWall => write!(f, "Optical object (square)"),
            Self::OpticalObjectRoundWall  => write!(f, "Optical object (round)"),
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LightPolarizationType {
    LinearHorizontal = 0,
    LinearVertical = 1,

    LinearDiagonal = 2,
    LinearAntiDiagonal = 3,

    CircularRightHand = 4,
    CircularLeftHand = 5,
    NotPolarized = 6
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PolarizerType {
    LinearHorizontal = 0,
    LinearVertical = 1,

    Linear45Degrees = 2,
    LinearTheta = 3,

    RightCircular = 4,
    LeftCircular = 5,

    QuarterWavePlateFastAxisVertical = 6,
    QuarterWavePlateFastAxisHorizontal = 7,
    QuarterWavePlateFastAxisTheta = 8,

    HalfWavePlateRotatedTheta = 9,
    HalfWavePlateFastAxisTheta = 10,

    GeneralWavePlateLinearRetarderTheta = 11,

    ArbitraryBirefringentMaterialTheta = 12
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Alignment {
    FRONT,
    RIGHT,
    UP
}

// Needed for the drop down list
impl Display for Alignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FRONT => write!(f, "Front"),
            Self::RIGHT => write!(f, "Right"),
            Self::UP => write!(f, "Up"),
        }
    }
}


// Needed for the drop down list
impl Display for LightPolarizationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LinearHorizontal => write!(f, "Linear horizontal"),
            Self::LinearVertical => write!(f, "Linear vertical"),

            Self::LinearDiagonal => write!(f, "Linear rotated 45 degrees"),
            Self::LinearAntiDiagonal => write!(f, "Linear rotated θ degrees"),

            Self::CircularRightHand => write!(f, "Right circular"),
            Self::CircularLeftHand => write!(f, "Left circular"),

            Self::NotPolarized => write!(f, "Not polarized")
        }
    }
}

// Needed for the drop down list
impl Display for PolarizerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LinearHorizontal => write!(f, "Linear horizontal"),
            Self::LinearVertical => write!(f, "Linear vertical"),

            Self::Linear45Degrees => write!(f, "Linear rotated 45 degrees"),
            Self::LinearTheta => write!(f, "Linear rotated θ degrees"),

            Self::RightCircular => write!(f, "Right circular"),
            Self::LeftCircular => write!(f, "Left circular"),

            Self::QuarterWavePlateFastAxisVertical => write!(f, "Quarter-wave plate with fast axis vertical"),
            Self::QuarterWavePlateFastAxisHorizontal => write!(f, "Quarter-wave plate with fast axis horizontal"),
            Self::QuarterWavePlateFastAxisTheta => write!(f, "Quarter-wave plate with fast axis at angle θ w.r.t the horizontal axis "),

            Self::HalfWavePlateRotatedTheta => write!(f, "Half-wave plate rotated by θ"),
            Self::HalfWavePlateFastAxisTheta => write!(f, "Half-wave plate with fast axis at angle θ w.r.t the horizontal axis"),

            Self::GeneralWavePlateLinearRetarderTheta => write!(f, "General Waveplate (Linear Phase Retarder)"),

            Self::ArbitraryBirefringentMaterialTheta => write!(f, "Arbitrary birefringent material (Elliptical phase retarder)"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldObject {
    // should add a way of discerning between gaussian beams and other types of lights
    pub object_type: ObjectType,
    pub rotation: [f32; 2],
    pub center: [f32; 3],
    pub color: Color32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub polarization: Vector2<Complex<f32>>,
    pub jones_matrix: Matrix2<Complex<f32>>,
    pub polarization_type: LightPolarizationType,

    // these next 3 should probably be an option for correctness
    pub aligned_to_object: usize,
    pub alignment: Alignment,
    pub aligned_distance: f32,
    // i want to make a vector but it doesn't implement copy
    // sometimes i regret ever trying to use rust at all
    // for now i'll just limit the amount of possibly aligned objects to 1
    pub object_aligned_to_self: usize
}

#[derive(Debug, Clone)]
pub struct World {
    pub hash_map: GPUHashTable,
    pub objects: [WorldObject; 166],
    pub aligned_objects: HashSet<usize>,
    // would be an array but i want to be able to use pop()
    // to remove an item but keep the memory contiguous
    pub light_sources: Vec<u32>,
    pub objects_stack: Vec<usize>,
    pub objects_associations: HashMap<usize, Vec<Vector3<u32>>>,
}

impl World {
    pub fn new() -> World {
        return World {
            hash_map: GPUHashTable::new(Vector3::new(200, 200, 200)),
            objects: [WorldObject::new(); 166],
            aligned_objects: HashSet::new(),
            light_sources: vec![],
            objects_stack: (1..166).collect(),
            objects_associations: HashMap::new()
        }
    }

    pub fn remove_object(&mut self, object_index: usize) {
        console::log_1(&format!("Positions occupied by object: {:?}", self.objects_associations.get(&object_index).unwrap()).into());

        self.aligned_objects.remove(&self.objects[object_index].object_aligned_to_self);
        // nasty but u know
        self.objects[self.objects[object_index].object_aligned_to_self].aligned_to_object = 0;

        console::log_1(&format!("aligned objects: {:?}", self.aligned_objects).into());

        for position_occupied_by_object in self.objects_associations.get(&object_index).unwrap() {
            match self.hash_map.remove(*position_occupied_by_object, object_index as u32) {
                Ok(()) => {}
                Err(e) => {console::log_1(&e.into())}
            }
        }

        for i in 0..self.light_sources.len() {
            if self.light_sources[i] == object_index as u32 {
                self.light_sources.remove(i);
                break;
            }
        }

        // we must also remove it from the objects list
        // and mark that space as available
        self.objects[object_index] = WorldObject::new();
        self.objects_stack.push(object_index);
        self.objects_associations.remove(&object_index);
    }

    // this should return an ok, in case the objects list is full and we can't add
    // anything here
    pub fn insert_object(&mut self, position: Vector3<i32>, object_definition: WorldObject) -> usize {
        // this line should possibly return an ok
        let available_index = self.objects_stack.pop().unwrap();
        let mut object_positions = vec![];

        match object_definition.object_type {
            ObjectType::CubeWall |
            ObjectType::OpticalObjectCube => {
                self.hash_map.insert( i32_to_u32_vec(position + Vector3::new(100, 100, 100)), available_index as u32);
                object_positions.push(i32_to_u32_vec(position + Vector3::new(100, 100, 100)));
            }

            ObjectType::LightSource => {
                let center = [object_definition.center[0] as u32, object_definition.center[1] as u32, object_definition.center[2] as u32];
                let truncated_radius = object_definition.radius as u32 + 1;

                for x in (center[0] - truncated_radius)..=(center[0] + truncated_radius) {
                    for y in (center[1] - truncated_radius)..=(center[1] + truncated_radius) {
                        for z in (center[2] - truncated_radius)..=(center[2] + truncated_radius) {
                            self.hash_map.insert(Vector3::new(x, y, z) + Vector3::new(100, 100, 100), available_index as u32);
                            object_positions.push(Vector3::new(x, y, z) + Vector3::new(100, 100, 100));
                        }
                    }
                }

                self.light_sources.push(available_index as u32);
            }

            ObjectType::RoundWall              |
            ObjectType::OpticalObjectRoundWall |
            ObjectType::SquareWall             |
            ObjectType::OpticalObjectSquareWall => {
                let center = [object_definition.center[0] as u32, object_definition.center[1] as u32, object_definition.center[2] as u32];
                let truncated_radius = object_definition.radius as u32 + 1;

                for x in (center[0] - truncated_radius)..=(center[0] + truncated_radius) {
                    for y in (center[1] - truncated_radius)..=(center[1] + truncated_radius) {
                        for z in (center[2] - truncated_radius)..=(center[2] + truncated_radius) {
                            self.hash_map.insert(Vector3::new(x, y, z) + Vector3::new(100, 100, 100), available_index as u32);
                            object_positions.push(Vector3::new(x, y, z) + Vector3::new(100, 100, 100));
                        }
                    }
                }
            }
        }

        // TODO this should change to a stack like with the
        // hashmap buckets
        // should handle the case when the object stack is full
        console::log_1(&format!("{:?}", available_index).into());
        self.objects_associations.insert(available_index, object_positions);
        self.objects[available_index] = object_definition;
        return available_index;
    }

    pub fn update_object_position(&mut self, object_index: usize, object_definition: WorldObject) {
        for position_occupied_by_object in self.objects_associations.get(&object_index).unwrap() {
            match self.hash_map.remove(*position_occupied_by_object, object_index as u32) {
                Ok(()) => {}
                Err(e) => {console::log_1(&e.into())}
            }
        }

        console::log_1(&format!("Removing object associations for index: {:?}", &object_index).into());
        self.objects_associations.remove(&object_index);

        let center = [object_definition.center[0] as u32, object_definition.center[1] as u32, object_definition.center[2] as u32];
        let truncated_radius = object_definition.radius as u32 + 1;
        let mut object_positions = vec![];

        // this can only be used on non cube objects
        for x in (center[0] - truncated_radius)..=(center[0] + truncated_radius) {
            for y in (center[1] - truncated_radius)..=(center[1] + truncated_radius) {
                for z in (center[2] - truncated_radius)..=(center[2] + truncated_radius) {
                    self.hash_map.insert(Vector3::new(x, y, z) + Vector3::new(100, 100, 100), object_index as u32);
                    object_positions.push(Vector3::new(x, y, z) + Vector3::new(100, 100, 100));
                }
            }
        }

        self.objects_associations.insert(object_index, object_positions);
    }

    pub fn get_gpu_compatible_world_objects_list(&self) -> Vec<u32> {
        self.objects.iter().flat_map(|object| {
            [
                object.object_type as u32,

                object.rotation[0].to_bits(),
                object.rotation[1].to_bits(),

                object.center[0].to_bits(),
                object.center[1].to_bits(),
                object.center[2].to_bits(),

                (object.color.r() as f32 / 255.0).to_bits(),
                (object.color.g() as f32 / 255.0).to_bits(),
                (object.color.b() as f32 / 255.0).to_bits(),

                object.width.to_bits(),
                object.height.to_bits(),

                object.radius.to_bits(),

                object.polarization[0].real().to_bits(),
                object.polarization[0].imaginary().to_bits(),

                object.polarization[1].real().to_bits(),
                object.polarization[1].imaginary().to_bits(),

                object.jones_matrix[0].real().to_bits(),
                object.jones_matrix[0].imaginary().to_bits(),

                object.jones_matrix[1].real().to_bits(),
                object.jones_matrix[1].imaginary().to_bits(),

                object.jones_matrix[2].real().to_bits(),
                object.jones_matrix[2].imaginary().to_bits(),

                object.jones_matrix[3].real().to_bits(),
                object.jones_matrix[3].imaginary().to_bits(),
            ]
        }).collect()
    }
}

impl WorldObject {
    pub fn new() -> WorldObject {
        return WorldObject {
            object_type: ObjectType::CubeWall,
            rotation: [0.0, PI / 2.0],

            center: [0.0, 0.0, 0.0],
            color: Color32::from_rgb(255, 0, 0),
            width: 0.5,
            height: 0.5,

            radius: 0.5,

            polarization: Vector2::new(Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)),
            jones_matrix: Matrix2::zeros(),

            polarization_type: LightPolarizationType::NotPolarized,

            aligned_to_object: 0,
            alignment: Alignment::FRONT,
            aligned_distance: 0.0,
            object_aligned_to_self: 0
        }
    }

    pub fn update_object_aligned_position(&mut self, aligned_to_object: &WorldObject) {
        let ray_dir: Vector3<f32>;

        match self.alignment {
            Alignment::FRONT => {
                let ray_dir_y = rotate3d_y(Vector3::new(0.0, 1.0, 0.0), aligned_to_object.rotation[0]);
                let ray_dir_x = rotate3d_x(Vector3::new(0.0, 0.0, 1.0), aligned_to_object.rotation[1]);
                ray_dir = (ray_dir_x + ray_dir_y).normalize() * self.aligned_distance;
            }

            Alignment::RIGHT => {
                let ray_dir_y = rotate3d_y(Vector3::new(0.0, 1.0, 0.0), aligned_to_object.rotation[0]);
                let ray_dir_x = rotate3d_x(Vector3::new(1.0, 0.0, 0.0), aligned_to_object.rotation[1]);
                ray_dir = (ray_dir_x + ray_dir_y).normalize() * self.aligned_distance;
            }

            Alignment::UP => {
                let ray_dir_y = rotate3d_y(Vector3::new(0.0, 1.0, 0.0), aligned_to_object.rotation[0]);
                let ray_dir_x = rotate3d_x(Vector3::new(0.0, 1.0, 0.0), aligned_to_object.rotation[1]);
                ray_dir = (ray_dir_x + ray_dir_y).normalize() * self.aligned_distance;
            }
        }

        self.center = [aligned_to_object.center[0] + ray_dir.x, aligned_to_object.center[1] + ray_dir.y, aligned_to_object.center[2] + ray_dir.z];

        self.center[0] = self.center[0].clamp(0.5, 24.5);
        self.center[1] = self.center[1].clamp(0.5, 24.5);
        self.center[2] = self.center[2].clamp(0.5, 24.5);
    }

    pub fn set_light_polarization(&mut self) {
        let type_of_object = self.polarization_type;
        match type_of_object {
            LightPolarizationType::NotPolarized => {
                self.polarization = Vector2::new(Complex::new(0.0, 0.0), Complex::new(0.0, 0.0))
            },

            LightPolarizationType::LinearHorizontal => {
                self.polarization = Vector2::new(Complex::new(1.0, 0.0), Complex::new(0.0, 0.0))
            },

            LightPolarizationType::LinearVertical => {
                self.polarization = Vector2::new(Complex::new(0.0, 0.0), Complex::new(1.0, 0.0))
            },

            LightPolarizationType::LinearDiagonal => {
                self.polarization = Vector2::new(Complex::new(1.0, 0.0), Complex::new(1.0, 0.0)).map(|x| x * (1.0 / (2.0).sqrt()));
            },

            LightPolarizationType::LinearAntiDiagonal => {
                self.polarization = Vector2::new(Complex::new(1.0, 0.0), Complex::new(-1.0, 0.0)).map(|x| x * (1.0 / (2.0).sqrt()));
            },

            LightPolarizationType::CircularRightHand => {
                self.polarization = Vector2::new(Complex::new(1.0, 0.0), Complex::new(0.0, -1.0)).map(|x| x * (1.0 / (2.0).sqrt()));
            },

            LightPolarizationType::CircularLeftHand => {
                self.polarization = Vector2::new(Complex::new(1.0, 0.0), Complex::new(0.0, 1.0)).map(|x| x * (1.0 / (2.0).sqrt()));
            },
        }
    }

    pub fn set_jones_matrix(&mut self, type_of_object: PolarizerType, angle: f32, relative_phase_retardation: f32, circularity: f32) {
        match type_of_object {
            PolarizerType::LinearHorizontal => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)
                )
            }

            PolarizerType::LinearVertical => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(0.0, 0.0), Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)
                )
            }

            PolarizerType::Linear45Degrees => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)
                ).map(|x| x * 0.5)
            }

            PolarizerType::LinearTheta => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(angle.cos().powi(2), 0.0),       Complex::new(angle.cos() * angle.sin(), 0.0),
                    Complex::new(angle.cos() * angle.sin(), 0.0), Complex::new(angle.sin().powi(2), 0.0)
                )
            }

            PolarizerType::RightCircular => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0, 1.0),
                    Complex::new(0.0,-1.0), Complex::new(1.0, 0.0)
                ).map(|x| x * 0.5)
            }

            PolarizerType::LeftCircular => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0,-1.0),
                    Complex::new(0.0, 1.0), Complex::new(1.0, 0.0)
                ).map(|x| x * 0.5)
            }

            PolarizerType::QuarterWavePlateFastAxisVertical => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0), Complex::new(0.0,-1.0)
                ).map(|x| x * Complex::new(0.0, PI / 4.0).exp())
            }

            PolarizerType::QuarterWavePlateFastAxisHorizontal => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0), Complex::new(0.0, 1.0)
                ).map(|x| x * Complex::new(0.0, -PI / 4.0).exp())
            }

            PolarizerType::QuarterWavePlateFastAxisTheta => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(angle.cos().powi(2), angle.sin().powi(2)),  (1f32 - Complex::new(0f32, -1f32)) * angle.sin() * angle.cos(),
                    (1f32 - Complex::new(0f32, -1f32)) * angle.sin() * angle.cos(), Complex::new(angle.sin().powi(2), angle.cos().powi(2))
                ).map(|x| x * Complex::new(0.0, -PI / 4.0).exp())
            }

            PolarizerType::HalfWavePlateRotatedTheta => {
                self.jones_matrix = Matrix2::new(
                    Complex::new((2.0 * angle).cos(), 0.0), Complex::new( (2.0 * angle).sin(), 1.0),
                    Complex::new((2.0 * angle).sin(), 1.0), Complex::new(-(2.0 * angle).cos(), 0.0)
                ).map(|x| x * 0.5)
            }

            PolarizerType::HalfWavePlateFastAxisTheta => {
                self.jones_matrix = Matrix2::new(
                    Complex::new(angle.cos().powi(2) - angle.sin().powi(2), 0.0), Complex::new(2.0 * angle.cos() * angle.sin(), 0.0),
                    Complex::new(2.0 * angle.cos() * angle.sin(), 0.0),           Complex::new(angle.sin().powi(2) - angle.cos().powi(2), 0.0)
                ).map(|x| x * Complex::new(0.0, -PI / 2.0).exp())
            }

            // god had no hand in creating these next 2
            PolarizerType::GeneralWavePlateLinearRetarderTheta => {
                let e_to_the_in = Complex::new(0.0, relative_phase_retardation).exp();

                self.jones_matrix = Matrix2::new(
                    angle.cos().powi(2)         + (e_to_the_in * angle.sin().powi(2)),
                    (angle.cos() * angle.sin()) - (e_to_the_in * angle.cos() * angle.sin()),

                    (angle.cos() * angle.sin()) - (e_to_the_in * angle.cos() * angle.sin()),
                    angle.sin().powi(2)         + (e_to_the_in * angle.cos().powi(2)),
                ).map(|x| x * Complex::new(0.0, -PI / 2.0).exp())
            }

            PolarizerType::ArbitraryBirefringentMaterialTheta => {
                let e_to_the_in   =      Complex::new(0.0, relative_phase_retardation).exp();
                let e_to_the_i_neg_phi = Complex::new(0.0,-circularity).exp();
                let e_to_the_i_phi =     Complex::new(0.0, circularity).exp();

                self.jones_matrix = Matrix2::new(
                     angle.cos().powi(2)                             + (e_to_the_in * angle.sin().powi(2)),
                    (e_to_the_i_neg_phi * angle.cos() * angle.sin()) - (e_to_the_in * e_to_the_i_neg_phi * angle.cos() * angle.sin()),

                    (e_to_the_i_phi * angle.cos() * angle.sin()) - (e_to_the_in * e_to_the_i_phi * angle.cos() * angle.sin()),
                     angle.sin().powi(2)          + (e_to_the_in * angle.cos().powi(2)),
                ).map(|x| x * Complex::new(0.0, -PI / 2.0).exp())
            }
        }
    }
}
