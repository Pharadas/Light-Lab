use std::{f32::consts::PI, fmt::{self, Display, Formatter}};
use nalgebra::{Complex, Matrix2};
use web_sys::console;
use math_vector::Vector;
use serde::{Deserialize, Serialize};

use crate::gpu_hash::GPUHashTable;

// WorldObject.type possible values
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ObjectType {
    CubeWall = 0,                   // Filled cube that can only be in uvec3 positions
    SquareWall = 1,                 // Infinitesimally thin square wall
    RoundWall = 2,                  // Infinitesimally thin round wall
    LightSource = 3,                // Sphere that represents a light source
    OpticalObjectCube = 4,          // An object represented using a jones matrix
    OpticalObjectSquareWall = 5,    // An object represented using a jones matrix
    OpticalObjectRoundWall = 6,     // An object represented using a jones matrix
}

#[derive(PartialEq)]
pub enum OpticalObject {
    LightSource,
    Polarizer_PhaseRetarder,
    Mirror,
    BeamSplitter,
    Wall
}

// Needed for the drop down list
impl Display for OpticalObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LightSource => write!(f, "Light source"),
            Self::Polarizer_PhaseRetarder => write!(f, "Polarizer"),
            Self::Mirror => write!(f, "Mirror"),
            Self::BeamSplitter => write!(f, "Beam splitter"),
            Self::Wall => write!(f, "Wall"),
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
struct Triangle {
    p0: Vector<f32>,
    p1: Vector<f32>,
    p2: Vector<f32>
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct ComplexNumber {
    dat: [f32; 2]
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Complex2x2Matrix {
    a: ComplexNumber,
    b: ComplexNumber,
    c: ComplexNumber,
    d: ComplexNumber
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Polarization {
    ex: ComplexNumber,
    ey: ComplexNumber
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldObject {
    // would use an enum but i want to keep compatibility
    // with the gpu version of this struct
    // would have written 'type' but it's a reserved keyword
    object_type: ObjectType,
    center: [f32; 3],
    top_left: [f32; 3],
    bottom_right: [f32; 3],
    radius: f32,
    polarization: Polarization,
    jones_matrix: Complex2x2Matrix
}

#[derive(Copy, Clone)]
enum Max {
    Steps(usize),
    Distance(f64),
}

#[derive(Debug, Clone)]
pub struct World {
    pub hash_map: GPUHashTable,
    objects: Vec<WorldObject>,
}

pub fn calculate_matrix(type_of_object: PolarizerType, angle: f32, relative_phase_retardation: f32, circularity: f32) -> Matrix2<Complex<f32>> {
    match type_of_object {
        PolarizerType::LinearHorizontal => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)
            )
        }

        PolarizerType::LinearVertical => {
            Matrix2::new(
                Complex::new(0.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)
            )
        }

        PolarizerType::Linear45Degrees => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)
            ).map(|x| x * 0.5)
        }

        PolarizerType::LinearTheta => {
            Matrix2::new(
                Complex::new(angle.cos().powi(2), 0.0),       Complex::new(angle.cos() * angle.sin(), 0.0),
                Complex::new(angle.cos() * angle.sin(), 0.0), Complex::new(angle.sin().powi(2), 0.0)
            )
        }

        PolarizerType::RightCircular => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0, 1.0),
                Complex::new(0.0,-1.0), Complex::new(1.0, 0.0)
            ).map(|x| x * 0.5)
        }

        PolarizerType::LeftCircular => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0,-1.0),
                Complex::new(0.0, 1.0), Complex::new(1.0, 0.0)
            ).map(|x| x * 0.5)
        }

        PolarizerType::QuarterWavePlateFastAxisVertical => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(0.0,-1.0)
            ).map(|x| x * Complex::new(0.0, PI / 4.0).exp())
        }

        PolarizerType::QuarterWavePlateFastAxisHorizontal => {
            Matrix2::new(
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(0.0, 1.0)
            ).map(|x| x * Complex::new(0.0, -PI / 4.0).exp())
        }

        PolarizerType::QuarterWavePlateFastAxisTheta => {
            Matrix2::new(
                Complex::new(angle.cos().powi(2), angle.sin().powi(2)),  (1f32 - Complex::new(0f32, -1f32)) * angle.sin() * angle.cos(),
                (1f32 - Complex::new(0f32, -1f32)) * angle.sin() * angle.cos(), Complex::new(angle.sin().powi(2), angle.cos().powi(2))
            ).map(|x| x * Complex::new(0.0, -PI / 4.0).exp())
        }

        PolarizerType::HalfWavePlateRotatedTheta => {
            Matrix2::new(
                Complex::new((2.0 * angle).cos(), 0.0), Complex::new( (2.0 * angle).sin(), 1.0),
                Complex::new((2.0 * angle).sin(), 1.0), Complex::new(-(2.0 * angle).cos(), 0.0)
            ).map(|x| x * 0.5)
        }

        PolarizerType::HalfWavePlateFastAxisTheta => {
            Matrix2::new(
                Complex::new(angle.cos().powi(2) - angle.sin().powi(2), 0.0), Complex::new(2.0 * angle.cos() * angle.sin(), 0.0),
                Complex::new(2.0 * angle.cos() * angle.sin(), 0.0),           Complex::new(angle.sin().powi(2) - angle.cos().powi(2), 0.0)
            ).map(|x| x * Complex::new(0.0, -PI / 2.0).exp())
        }

        // god had no hand in creating these next 2
        PolarizerType::GeneralWavePlateLinearRetarderTheta => {
            let e_to_the_in = Complex::new(0.0, relative_phase_retardation).exp();

            Matrix2::new(
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

            Matrix2::new(
                 angle.cos().powi(2)                             + (e_to_the_in * angle.sin().powi(2)),
                (e_to_the_i_neg_phi * angle.cos() * angle.sin()) - (e_to_the_in * e_to_the_i_neg_phi * angle.cos() * angle.sin()),

                (e_to_the_i_phi * angle.cos() * angle.sin()) - (e_to_the_in * e_to_the_i_phi * angle.cos() * angle.sin()),
                 angle.sin().powi(2)          + (e_to_the_in * angle.cos().powi(2)),
            ).map(|x| x * Complex::new(0.0, -PI / 2.0).exp())
        }
    }
}

fn to_f64_slice(a: Vector<f32>) -> [f64; 3] {
    return [a.x as f64, a.y as f64, a.z as f64];
}

// Thanks to https://github.com/leroycep/ascii-raycaster/blob/master/src/main.rs
fn raymarch(pos: [f64; 3], dir: [f64; 3], end_pos: [f64; 3], max: Max) -> Vec<Vector<i32>> {
    let mut tiles_found: Vec<Vector<i32>> = vec![];

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

    let mut last_distance = (Vector::new(map_pos[0], map_pos[1], map_pos[2]) - Vector::new(end_pos[0], end_pos[1], end_pos[2])).length();

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

impl World {
    pub fn new() -> World {
        // let sample_triangle = Triangle {
        //     p0: Vector::new(5.3,   5.3, 5.3),
        //     p1: Vector::new(-5.3,  5.3, -5.3),
        //     p2: Vector::new(-5.3, -5.3, 5.3)
        // };

        // let mut gpu_hash = GPUHashTable::new(Vector::new(200, 200, 200));

        // let a_through_b_rasterized = raymarch(to_f64_slice(sample_triangle.p0), to_f64_slice(sample_triangle.p1 - sample_triangle.p0), to_f64_slice(sample_triangle.p1), Max::Steps(50));

        // console::log_1(&format!("final list: {:?}", a_through_b_rasterized).into());

        // for position in a_through_b_rasterized {
        //     gpu_hash.insert((position + Vector::new(100, 100, 100)).as_u32s(), 1);
        //     // now just keep firing rays to every position and rasterizing
        //     let c_through_position_rasterized = raymarch(to_f64_slice(sample_triangle.p2), to_f64_slice(position.as_f32s() - sample_triangle.p2), to_f64_slice(position.as_f32s()), Max::Steps(50));
        //     console::log_1(&format!("final list inside loop: {:?}", c_through_position_rasterized).into());

        //     // just put it into the grid
        //     for new_position in c_through_position_rasterized {
        //         gpu_hash.insert((new_position + Vector::new(100, 100, 100)).as_u32s(), 1);
        //     }
        // }

        // gpu_hash.insert(Vector::new(100u32, 100u32, 100u32), 1);

        return World {
            hash_map: GPUHashTable::new(Vector::new(200, 200, 200)),
            objects: vec![],
        }
    }

    pub fn insert_object(&mut self, position: Vector<i32>, object_definition: WorldObject) {
        self.hash_map.insert((position + Vector::new(100, 100, 100)).as_u32s(), self.objects.len() as u32);
        self.objects.push(object_definition);
    }

    pub fn get_gpu_compatible_world_objects_list(&self) -> Vec<u32> {
        self.objects.iter().flat_map(|object| {
            [
                object.object_type as u32,

                object.center[0].to_bits(),
                object.center[1].to_bits(),
                object.center[2].to_bits(),

                object.top_left[0].to_bits(),
                object.top_left[1].to_bits(),
                object.top_left[2].to_bits(),

                object.bottom_right[0].to_bits(),
                object.bottom_right[1].to_bits(),
                object.bottom_right[2].to_bits(),

                object.top_left[0].to_bits(),
                object.top_left[1].to_bits(),
                object.top_left[2].to_bits(),

                object.radius.to_bits(),

                object.polarization.ex.dat[0].to_bits(),
                object.polarization.ex.dat[1].to_bits(),

                object.polarization.ey.dat[0].to_bits(),
                object.polarization.ey.dat[1].to_bits(),

                object.jones_matrix.a.dat[0].to_bits(),
                object.jones_matrix.a.dat[1].to_bits(),

                object.jones_matrix.b.dat[0].to_bits(),
                object.jones_matrix.b.dat[1].to_bits(),

                object.jones_matrix.c.dat[0].to_bits(),
                object.jones_matrix.c.dat[1].to_bits(),

                object.jones_matrix.d.dat[0].to_bits(),
                object.jones_matrix.d.dat[1].to_bits(),
            ]
        }).collect()
    }
}

impl WorldObject {
    pub fn new(object_type: ObjectType) -> WorldObject {
        return WorldObject {
            object_type: ObjectType::CubeWall,

            center: [0.0, 0.0, 0.0],
            top_left: [0.0, 0.0, 0.0],
            bottom_right: [0.0, 0.0, 0.0],

            radius: 0.0,

            polarization: Polarization {
                ex: ComplexNumber { dat: [0.0, 0.0] }, 
                ey: ComplexNumber { dat: [0.0, 0.0] }
            },

            jones_matrix: Complex2x2Matrix {
                a: ComplexNumber { dat: [0.0, 0.0] }, 
                b: ComplexNumber { dat: [0.0, 0.0] }, 
                c: ComplexNumber { dat: [0.0, 0.0] }, 
                d: ComplexNumber { dat: [0.0, 0.0] }
            }
        }
    }
}