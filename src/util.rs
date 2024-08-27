use nalgebra::Vector3;

pub fn to_f64_slice(a: Vector3<f32>) -> [f64; 3] {
    return [a.x as f64, a.y as f64, a.z as f64];
}

pub fn i32_to_f32_vec(in_val: Vector3<i32>) -> Vector3<f32> {
    return Vector3::new(in_val.x as f32, in_val.y as f32, in_val.z as f32);
}

pub fn i32_to_u32_vec(in_val: Vector3<i32>) -> Vector3<u32> {
    return Vector3::new(in_val.x as u32, in_val.y as u32, in_val.z as u32);
}

pub fn f32_slice_to_u32_vec(in_val: [f32; 3]) -> Vector3<u32> {
    return Vector3::new(in_val[0] as u32, in_val[1] as u32, in_val[2] as u32);
}

