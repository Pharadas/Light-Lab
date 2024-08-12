precision mediump float;
precision mediump int;
in vec4 v_color;

uniform vec2 u_rotation;
uniform vec3 position; 
uniform uint objects[3000];
uniform uint buckets[1000];

out vec4 out_color;

// Constants definitions =================================
const int MAX_RAY_STEPS = 1000;
const ivec3 WORLD_SIZE = ivec3(200, 200, 200);
const float PI = 3.1416;
const uint U32_MAX = uint(4294967295);

// Struct definitions ====================================
struct RayObject {
  // current direction of the ray
  vec3 dir;
  // original position of the ray
  vec3 pos;
  // current position of the ray truncated
  ivec3 map_pos;

  vec3 delta_dist;
  ivec3 step;
  vec3 side_dist; 

  // current axis in which the ray entered the block
  bvec3 mask;

  float distance_traveled;
  vec3 current_real_position;
  vec3 original_dir;

  bool ended_in_hit;

  vec4 color;
  // Complex2x2Matrix optical_objects_found_product;
  // int optical_objects_through_which_it_passed;
};

// Math utils functions =================================
vec3 rotate3dY(vec3 v, float a) {
    float cosA = cos(a);
    float sinA = sin(a);
    return vec3(
        v.x * cosA + v.z * sinA,
        v.y,
        -v.x * sinA + v.z * cosA
    );
}

vec3 rotate3dX(vec3 v, float a) {
    float cosA = cos(a);
    float sinA = sin(a);
    return vec3(
        v.x,
        v.y * cosA - v.z * sinA,
        v.y * sinA + v.z * cosA
    );
}

// Hash implementation
uint hash(ivec3 val) {
  return uint(val.x + WORLD_SIZE.y * (val.y + WORLD_SIZE.z * val.z));
}

// Ray marching code
void step_ray(inout RayObject ray) {
  ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
  ray.side_dist += vec3(ray.mask) * ray.delta_dist;
  ray.map_pos += ivec3(vec3(ray.mask)) * ray.step;
}

// the ray will simply iterate over the space in the direction it's facing trying to hit a 'solid' object
// Walls: will stop
// Models: will stop
// Mirrors: will bounce off the mirror and continue iterating
// Light source: will stop
// Optical Object: will multiply it's internal jones matrix and continue iterating
void iterateRayInDirection(inout RayObject ray) {
  for (int i = 0; i < MAX_RAY_STEPS; i += 1) {
    step_ray(ray);
    uint hashed_value = hash(ray.map_pos + ivec3(100, 100, 100));
    uint original_index = hashed_value % uint(1000);
    uint current_index = buckets[original_index];

    while (objects[(current_index * uint(3)) + uint(2)] != U32_MAX) {
      if (objects[current_index * uint(3)] == hashed_value) {
        ray.ended_in_hit = true;
        return;
      }

      current_index = objects[(current_index * uint(3)) + uint(2)];
    }

    if (objects[current_index * uint(3)] == hashed_value) {
      ray.ended_in_hit = true;
      return;
    }

    if ((ray.map_pos.x > 100 || ray.map_pos.x <= 0) || 
        (ray.map_pos.y > 100 || ray.map_pos.y <= 0) ||
        (ray.map_pos.z > 100 || ray.map_pos.z <= 0)
    ) {
      ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
      ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;

      float h = 2.0 + checker(ray.current_real_position);
      ray.color *= vec4(h, h, h, 1);

      ray.ended_in_hit = true;
      return;
    }

    step_ray(ray);
  }
}

void main() {
  vec2 viewport_dimensions = vec2(750., 750.);
  vec2 screen_pos = ((gl_FragCoord.xy / viewport_dimensions) * 2.) - 1.;

  vec3 camera_dir = vec3(0.0, 0.0, 1.0);
  vec3 camera_plane_u = vec3(1.0, 0.0, 0.0);
  vec3 camera_plane_v = vec3(0.0, 1.0, 0.0);

  vec3 ray_dir = camera_dir + screen_pos.x * camera_plane_u + screen_pos.y * camera_plane_v;
  ray_dir = rotate3dX(ray_dir, u_rotation.y);
  ray_dir = rotate3dY(ray_dir, u_rotation.x);
  ray_dir = normalize(ray_dir);

  RayObject ray;
    ray.dir = ray_dir;
    ray.pos = position;
    ray.map_pos = ivec3(ray.pos);
    ray.delta_dist = 1.0 / abs(ray.dir);
    ray.step = ivec3(sign(ray.dir));
    ray.side_dist = (sign(ray.dir) * (vec3(ray.map_pos) - ray.pos) + (sign(ray.dir) * 0.5) + 0.5) * ray.delta_dist;
    ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
    ray.color = vec4(1.0);
    ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
    ray.current_real_position = ray.pos + ray.dir * length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
    ray.ended_in_hit = false;
    // ray.optical_objects_through_which_it_passed = 0;

  iterateRayInDirection(ray);

  if (ray.ended_in_hit) {
    out_color = vec4(ray.mask, 1.0) * ray.color;
  } else {
    out_color = vec4(ray.color.x, ray_dir.y, ray_dir.z, 1.);
  }
}
