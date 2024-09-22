use std::{cmp::max, collections::HashMap, f32::consts::PI, fmt::{self, Display, Formatter}, u32};
use egui::{Color32, TextBuffer};
use nalgebra::{Complex, ComplexField, Matrix2, Vector3};
use web_sys::console;
use serde::{Deserialize, Serialize};

use crate::{gpu_hash::GPUHashTable, util::{f32_slice_to_u32_vec, i32_to_f32_vec, i32_to_u32_vec, to_f64_slice}};

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
pub struct Polarization {
    ex: Complex<f32>,
    ey: Complex<f32>
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
    pub polarization: Polarization,
    pub jones_matrix: Matrix2<Complex<f32>>
}

#[derive(Copy, Clone)]
enum Max {
    Steps(usize),
    Distance(f64),
}

#[derive(Debug, Clone)]
pub struct World {
    pub hash_map: GPUHashTable,
    pub objects: [WorldObject; 166],
    // would be an array but i want to be able to use pop()
    // to remove an item but keep the memory contiguous
    pub light_sources: Vec<u32>,
    objects_stack: Vec<usize>,
    objects_associations: HashMap<usize, Vec<Vector3<u32>>>,
}

// Thanks to https://github.com/leroycep/ascii-raycaster/blob/master/src/main.rs
fn raymarch(pos: [f64; 3], dir: [f64; 3], end_pos: [f64; 3], max: Max) -> Vec<Vector3<i32>> {
    let mut tiles_found: Vec<Vector3<i32>> = vec![];

    let (max_steps, _max_distance) = match max {
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
    let mut _side;
    for i in 0..3 {
        if dir[i] < 0.0 {
            step[i] = -1.0;
            side_dist[i] = (pos[i] - map_pos[i]) * delta_dist[i];
        } else {
            step[i] = 1.0;
            side_dist[i] = (map_pos[i] + 1.0 - pos[i]) * delta_dist[i];
        }
    }

    let mut last_distance = (Vector3::new(map_pos[0], map_pos[1], map_pos[2]) - Vector3::new(end_pos[0], end_pos[1], end_pos[2])).magnitude();

    for _ in 0..max_steps {
        if side_dist[0] < side_dist[1] && side_dist[0] < side_dist[2] {
            side_dist[0] += delta_dist[0];
            map_pos[0] += step[0];
            _side = 1;
        } else if side_dist[1] < side_dist[2] {
            side_dist[1] += delta_dist[1];
            map_pos[1] += step[1];
            _side = 3;
        } else {
            side_dist[2] += delta_dist[2];
            map_pos[2] += step[2];
            _side = 2;
        }
        tiles_found.push(Vector3::new(map_pos[0] as i32, map_pos[1] as i32, map_pos[2] as i32));

        if (Vector3::new(map_pos[0], map_pos[1], map_pos[2]) - Vector3::new(end_pos[0], end_pos[1], end_pos[2])).magnitude() > last_distance { // check that we are getting closer
            console::log_1(&"exited ray caster when ray passed target".into());
            return tiles_found;
        }

        last_distance = (Vector3::new(map_pos[0], map_pos[1], map_pos[2]) - Vector3::new(end_pos[0], end_pos[1], end_pos[2])).magnitude();

        if map_pos[0] as i32 == end_pos[0] as i32 && map_pos[1] as i32 == end_pos[1] as i32 && map_pos[2] as i32 == end_pos[2] as i32 {
            console::log_1(&"exited ray caster normally".into());
            return tiles_found;
        }
    }
    return tiles_found;
}

impl World {
    pub fn new() -> World {
        return World {
            hash_map: GPUHashTable::new(Vector3::new(200, 200, 200)),
            objects: [WorldObject::new(); 166],
            light_sources: vec![],
            objects_stack: (1..166).collect(),
            objects_associations: HashMap::new()
        }
    }

    pub fn remove_object(&mut self, object_index: usize) {
        console::log_1(&format!("Positions occupied by object: {:?}", self.objects_associations.get(&object_index).unwrap()).into());
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

    }

    // this should return an ok, in case the objects list is full and we can't add
    // anything here
    pub fn insert_object(&mut self, position: Vector3<i32>, object_definition: WorldObject) {
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
                let truncated_radius = object_definition.height.max(object_definition.width) as u32 + 1;

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

                object.polarization.ex.real().to_bits(),
                object.polarization.ex.imaginary().to_bits(),

                object.polarization.ey.real().to_bits(),
                object.polarization.ey.imaginary().to_bits(),

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

            polarization: Polarization {
                ex: Complex::new(0.0, 0.0),
                ey: Complex::new(0.0, 0.0)
            },

            jones_matrix: Matrix2::zeros()
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
