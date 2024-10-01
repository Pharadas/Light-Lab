use nalgebra::Vector3;

pub fn i32_to_u32_vec(in_val: Vector3<i32>) -> Vector3<u32> {
    return Vector3::new(in_val.x as u32, in_val.y as u32, in_val.z as u32);
}

