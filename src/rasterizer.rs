use web_sys::console;
use math_vector::Vector;

use crate::gpu_hash::{self, GPUHashTable};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct World {
    pub hash_map: GPUHashTable,
    // objects: Object,
    triangle: Triangle
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

impl World {
    pub fn new() -> World {
        let sample_triangle = Triangle {
            p0: Vector::new(5.3,   5.3, 5.3),
            p1: Vector::new(-5.3,  5.3, -5.3),
            p2: Vector::new(-5.3, -5.3, 5.3)
        };

        let mut gpu_hash = GPUHashTable::new(Vector::new(200, 200, 200));

        let a_through_b_rasterized = raymarch(to_f64_slice(sample_triangle.p0), to_f64_slice(sample_triangle.p1 - sample_triangle.p0), to_f64_slice(sample_triangle.p1), Max::Steps(50));

        console::log_1(&format!("final list: {:?}", a_through_b_rasterized).into());

        for position in a_through_b_rasterized {
            gpu_hash.insert((position + Vector::new(100, 100, 100)).as_u32s(), 1);
            // now just keep firing rays to every position and rasterizing
            let c_through_position_rasterized = raymarch(to_f64_slice(sample_triangle.p2), to_f64_slice(position.as_f32s() - sample_triangle.p2), to_f64_slice(position.as_f32s()), Max::Steps(50));
            console::log_1(&format!("final list inside loop: {:?}", c_through_position_rasterized).into());

            // just put it into the grid
            for new_position in c_through_position_rasterized {
                gpu_hash.insert((new_position + Vector::new(100, 100, 100)).as_u32s(), 1);
            }
        }

        gpu_hash.insert(Vector::new(100u32, 100u32, 100u32), 1);

        console::log_1(&format!("gpu hash: {:?}", gpu_hash).into());

        return World {
            hash_map: GPUHashTable::new(Vector::new(200, 200, 200)),
            triangle: sample_triangle
        }
    }

    pub fn insert_cube(&mut self, position: Vector<i32>) {
        self.hash_map.insert((position + Vector::new(100, 100, 100)).as_u32s(), 1);
    }

    pub fn rebuild() {

    }
}
