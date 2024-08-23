precision mediump float;
precision mediump int;
in vec4 v_color;

uniform vec2 u_rotation;
uniform vec3 position; 
uniform uint objects[3000];
uniform uint buckets[1000];
uniform vec2 viewport_dimensions;
uniform float time;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 object_found;

// Constants definitions =================================
const int MAX_RAY_STEPS = 1000;
const ivec3 WORLD_SIZE = ivec3(200, 200, 200);
const float PI = 3.1416;
const uint U32_MAX = uint(4294967295);

// WorldObject.type possible values
const uint CUBE_WALL = uint(0);                   // Filled cube that can only be in uvec3 positions
const uint SQUARE_WALL = uint(1);                 // Infinitesimally thin square wall
const uint ROUND_WALL = uint(2);                  // Infinitesimally thin round wall
const uint LIGHT_SOURCE = uint(3);                // Sphere that represents a light source
const uint OPTICAL_OBJECT_CUBE = uint(4);         // An object represented using a jones matrix
const uint OPTICAL_OBJECT_SQUARE_WALL = uint(5);  // An object represented using a jones matrix
const uint OPTICAL_OBJECT_ROUND_WALL = uint(6);   // An object represented using a jones matrix

struct gsl_complex {
  float dat[2];
};

struct ComplexNumber {
  vec2 dat; // 64 bits
};

// Complex matrix =
// |a b|
// |c d|
struct Complex2x2Matrix { // 256 bits
  ComplexNumber a; // 64 bits
  ComplexNumber b; // 64 bits
  ComplexNumber c; // 64 bits
  ComplexNumber d; // 64 bits
};

struct Polarization { // 128 bits
  ComplexNumber Ex; // 64 bits
  ComplexNumber Ey; // 64 bits
};

// Struct definitions ====================================
struct WorldObject { // 736 bits -> 23 bytes
  uint type; // 32 bits
  vec3 center; // 96 bits
  // top_left and bottom_right will only be relevant if the object is a
  // square of round wall
  vec3 top_left; // 96 bits
  vec3 bottom_right; // 96 bits
  // Will only be relevant if the object is a round wall
  // or a light source
  // will be the radius of the sphere if it's a light source
  // and the radius of the wall
  float radius; // 32 bits
  // Will only be relevant if it's a light source
  Polarization polarization; // 128 bits
  // Will only be relevant if it's an optical object
  Complex2x2Matrix jones_matrix; // 256 bits
};

uniform WorldObject objects_definitions[100];

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
  uint object_hit;
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

float checker(vec3 p) {
  float t = 10.0;
  return step(0.0, sin(PI * p.x + PI/t)*sin(PI *p.y + PI/t)*sin(PI *p.z + PI/t));
}

// Hash implementation
uint hash(ivec3 val) {
  return uint(val.x + WORLD_SIZE.y * (val.y + WORLD_SIZE.z * val.z));
}

// Intersections code
// Thanks to iq's https://www.shadertoy.com/view/XtlBDs
// 0--b--3
// |\
// a c
// |  \
// 1    2
//
vec3 quadIntersect( in vec3 ro, in vec3 rd, in vec3 v0, in vec3 v1, in vec3 v2, in vec3 v3 ) {
    // lets make v0 the origin
    vec3 a = v1 - v0;
    vec3 b = v3 - v0;
    vec3 c = v2 - v0;
    vec3 p = ro - v0;

    // intersect plane
    vec3 nor = cross(a,b);
    float t = -dot(p,nor)/dot(rd,nor);
    if( t<0.0 ) return vec3(-1.0);

    // intersection point
    return p + t*rd;
}

float computeDistance(vec3 A, vec3 B, vec3 C) {
	float x = length(cross(B - A, C - A));
	float y = length(B - A);
	return x / y;
}

// Ray marching code
void step_ray(inout RayObject ray) {
  ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
  ray.side_dist += vec3(ray.mask) * ray.delta_dist;
  ray.map_pos += ivec3(vec3(ray.mask)) * ray.step;
}

void get_object_hit(uint object_index, inout RayObject ray) {
  WorldObject selected_object = objects_definitions[object_index];

  if (selected_object.type == CUBE_WALL) {
    ray.ended_in_hit = true;
    return;
  }
}

// the ray will simply iterate over the space in the direction it's facing trying to hit a 'solid' object
// Walls: will stop
// Models: will stop
// Mirrors: will bounce off the mirror and continue iterating
// Light source: will stop
// Optical Object: will multiply it's internal jones matrix and continue iterating
void iterateRayInDirection(inout RayObject ray) {
  for (int i = 0; i < MAX_RAY_STEPS; i += 1) {
    uint hashed_value = hash(ray.map_pos + ivec3(100, 100, 100));
    uint original_index = hashed_value % uint(1000);
    uint current_index = buckets[original_index];

    vec3 quad_hit_position = quadIntersect(
      ray.pos, 
      ray.dir, 
      vec3(49.9, 49.4, 51.2), 
      vec3(49.5, 49.2, 51.2), 
      vec3(49.5, 49.2, 51.8), 
      vec3(49.5, 49.8, 51.8)
    );

    // TEMPORAL check if it hits a random plane
    if (ray.map_pos == ivec3(49, 49, 51) && (length(quad_hit_position) < 0.2)) {
      ray.object_hit = uint(0);

      float h = 2.0 + checker(quad_hit_position * 10.0);
      ray.color *= vec4(h, h, h, 1);
      ray.object_hit = uint(1);

      ray.ended_in_hit = true;
      return;
    }

    // search the item in the "linked list"
    while (objects[(current_index * uint(3)) + uint(2)] != U32_MAX) {
      if (objects[current_index * uint(3)] == hashed_value) {
        get_object_hit(current_index + uint(1), ray);
        ray.object_hit = current_index + uint(1);
        return;
      }

      current_index = objects[(current_index * uint(3)) + uint(2)];
    }

    if (objects[current_index * uint(3)] == hashed_value) {
      get_object_hit(objects[current_index * uint(3) + uint(1)], ray);
      ray.object_hit = objects[current_index * uint(3) + uint(1)]; // key.val
      return;
    }

    if ((ray.map_pos.x > 100 || ray.map_pos.x <= 0) || 
        (ray.map_pos.y > 100 || ray.map_pos.y <= 0) ||
        (ray.map_pos.z > 100 || ray.map_pos.z <= 0)
    ) {
      ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
      ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;
      ray.object_hit = uint(0);

      float h = 2.0 + checker(ray.current_real_position);
      ray.color *= vec4(h, h, h, 1);

      ray.ended_in_hit = true;
      return;
    }

    step_ray(ray);
  }
}

void main() {
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

  object_found = vec4(float(ray.object_hit) / 255.0, 0.0, 0.0, 0.0);

  if (ray.ended_in_hit) {
    out_color = vec4(vec3(ray.mask) * 0.2, 1.0) * ray.color;
  } else {
    out_color = vec4(ray.color.x, ray_dir.y, ray_dir.z, 1.);
  }
}
