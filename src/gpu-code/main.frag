precision mediump float;
precision mediump int;
in vec4 v_color;

uniform vec2 u_rotation;
uniform vec3 position; 
uniform vec2 viewport_dimensions;
uniform float time;
uniform float cube_scaling_factor;
uniform uint light_sources_count;
uniform float background_light_min;

uniform uint lights_definitions_indices[166];
uniform uint objects[3000];
uniform uint buckets[1000];
// to be able to use WorldObject objects_definitions[] i'd have to have
// sent it in a compatible alignment, not doin that tho
uniform uint objects_definitions[4000];

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

const uint OBJECT_SIZE = uint(24);

// Complex matrix =
// |a b|
// |c d|
struct Complex2x2Matrix { // 256 bits
  vec2 a; // 64 bits
  vec2 b; // 64 bits
  vec2 c; // 64 bits
  vec2 d; // 64 bits
};

struct Polarization { // 128 bits
  vec2 Ex; // 64 bits
  vec2 Ey; // 64 bits
};

// Struct definitions ====================================
struct WorldObject {
  uint type;
  vec2 rotation;
  vec3 center;
  vec3 color;
  float width;
  float height;
  // Will only be relevant if the object is a round wall
  // or a light source
  // will be the radius of the sphere if it's a light source
  // and the radius of the wall
  float radius;
  // Will only be relevant if it's a light source
  Polarization polarization;
  // Will only be relevant if it's an optical object
  Complex2x2Matrix jones_matrix;
};

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

struct ObjectGoal {
  WorldObject goal;
  uint goal_index;
  bool has_goal;
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

float computeDistance(vec3 A, vec3 B, vec3 C) {
	float x = length(cross(B - A, C - A));
	float y = length(B - A);
	return x / y;
}

float checker(vec3 p) {
  float t = 1.0;
  return step(0.0, sin(PI * p.x + PI/t)*sin(PI *p.y + PI/t)*sin(PI *p.z + PI/t));
}

// Complex math utils functions ========================
// Have to define it here because we may use it to define the phase retarder
// Complex Number math by julesb
// https://github.com/julesb/glsl-util
// Additions by Johan Karlsson (DonKarlssonSan)

vec2 cx_exp (vec2 z) {
	return exp(z.x) * vec2(cos(z.y), sin(z.y));
}

#define cx_mul(a, b) vec2(a.x*b.x-a.y*b.y, a.x*b.y+a.y*b.x)
#define cx_div(a, b) vec2(((a.x*b.x+a.y*b.y)/(b.x*b.x+b.y*b.y)),((a.y*b.x-a.x*b.y)/(b.x*b.x+b.y*b.y)))
#define cx_modulus(a) length(a)
#define cx_conj(a) vec2(a.x, -a.y)
#define cx_arg(a) atan(a.y, a.x)
#define cx_sin(a) vec2(sin(a.x) * cosh(a.y), cos(a.x) * sinh(a.y))
#define cx_cos(a) vec2(cos(a.x) * cosh(a.y), -sin(a.x) * sinh(a.y))

vec2 cx_sqrt(vec2 a) {
  float r = length(a);
  float rpart = sqrt(0.5*(r+a.x));
  float ipart = sqrt(0.5*(r-a.x));
  if (a.y < 0.0) ipart = -ipart;
  return vec2(rpart,ipart);
}

vec2 cx_tan(vec2 a) {return cx_div(cx_sin(a), cx_cos(a)); }

vec2 cx_log(vec2 a) {
    float rpart = sqrt((a.x*a.x)+(a.y*a.y));
    float ipart = atan(a.y,a.x);
    if (ipart > PI) ipart=ipart-(2.0*PI);
    return vec2(log(rpart),ipart);
}

vec2 cx_mobius(vec2 a) {
    vec2 c1 = a - vec2(1.0,0.0);
    vec2 c2 = a + vec2(1.0,0.0);
    return cx_div(c1, c2);
}

vec2 cx_z_plus_one_over_z(vec2 a) {
    return a + cx_div(vec2(1.0,0.0), a);
}

vec2 cx_z_squared_plus_c(vec2 z, vec2 c) {
    return cx_mul(z, z) + c;
}

vec2 cx_sin_of_one_over_z(vec2 z) {
    return cx_sin(cx_div(vec2(1.0,0.0), z));
}

////////////////////////////////////////////////////////////
// end Complex Number math by julesb
////////////////////////////////////////////////////////////

// My own additions to complex number math
#define cx_sub(a, b) vec2(a.x - b.x, a.y - b.y)
#define cx_add(a, b) vec2(a.x + b.x, a.y + b.y)
#define cx_abs(a) length(a)
vec2 cx_to_polar(vec2 a) {
    float phi = atan(a.y / a.x);
    float r = length(a);
    return vec2(r, phi); 
}
    
// Complex power
// Let z = r(cos θ + i sin θ)
// Then z^n = r^n (cos nθ + i sin nθ)
vec2 cx_pow(vec2 a, float n) {
    float angle = atan(a.y, a.x);
    float r = length(a);
    float real = pow(r, n) * cos(n*angle);
    float im = pow(r, n) * sin(n*angle);
    return vec2(real, im);
}

// NOTE
// matrix =
// [a b
//  c d]
Complex2x2Matrix cx_2x2_mat_mul(Complex2x2Matrix A, Complex2x2Matrix B) {
  Complex2x2Matrix resultant_mat;
  resultant_mat.a = A.a * B.a + A.b * B.c;
  resultant_mat.b = A.a * B.b + A.b * B.d;
  resultant_mat.c = A.c * B.a + A.d * B.c;
  resultant_mat.d = A.c * B.b + A.d * B.d;

  return resultant_mat;
}

Complex2x2Matrix cx_scalar_x_2x2_mat_mul(vec2 cx_scalar, Complex2x2Matrix cx_mat) {
  cx_mat.a = cx_mul(cx_scalar, cx_mat.a);
  cx_mat.b = cx_mul(cx_scalar, cx_mat.b);
  cx_mat.c = cx_mul(cx_scalar, cx_mat.c);
  cx_mat.d = cx_mul(cx_scalar, cx_mat.d);

  return cx_mat;
}

// TODO: maybe make a cx vec2 so that this is more general
Polarization cx_2x2_mat_x_cx_pol_mul(Complex2x2Matrix mat, Polarization vec) {
  Polarization result = Polarization(vec2(0, 0), vec2(0, 0));
  result.Ex = cx_add(cx_mul(mat.a, vec.Ex), cx_mul(mat.b, vec.Ey));
  result.Ey = cx_add(cx_mul(mat.c, vec.Ey), cx_mul(mat.d, vec.Ey));

  return result;
}

// Hash implementation
uint hash(ivec3 val) {
  return uint(val.x + WORLD_SIZE.y * (val.y + WORLD_SIZE.z * val.z));
}

WorldObject get_object_at_index(uint object_index) {
  WorldObject selected_object;
    // this whole section could break shit,
    // should add a check here or before sending
    selected_object.type = objects_definitions[object_index * OBJECT_SIZE];

    selected_object.rotation.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(1)]);
    selected_object.rotation.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(2)]);

    selected_object.center.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(3)]);
    selected_object.center.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(4)]);
    selected_object.center.z = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(5)]);

    selected_object.color.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(6)]);
    selected_object.color.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(7)]);
    selected_object.color.z = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(8)]);

    selected_object.width = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(9)]);
    selected_object.height = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(10)]);

    selected_object.radius = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(11)]);

    selected_object.polarization.Ex.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(12)]);
    selected_object.polarization.Ex.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(13)]);

    selected_object.polarization.Ey.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(14)]);
    selected_object.polarization.Ey.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(15)]);

    selected_object.jones_matrix.a.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(16)]);
    selected_object.jones_matrix.a.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(17)]);

    selected_object.jones_matrix.b.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(18)]);
    selected_object.jones_matrix.b.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(19)]);

    selected_object.jones_matrix.c.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(20)]);
    selected_object.jones_matrix.c.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(21)]);

    selected_object.jones_matrix.d.x = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(22)]);
    selected_object.jones_matrix.d.y = uintBitsToFloat(objects_definitions[(object_index * OBJECT_SIZE) + uint(23)]);

    return selected_object;
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

// s -> ray start, c -> sphere center, d -> ray direction, r -> sphere radius
vec3 raySphereIntersectPos(vec3 s, vec3 c, vec3 d, float r) {
  // Calculate ray start's offset from the sphere center
  vec3 p = s - c;

  float rSquared = r * r;
  float p_d = dot(p, d);

  // The sphere is behind or surrounding the start point.
  if(p_d > 0.0 || dot(p, p) < rSquared) {
    return vec3(-1.0);
  }

  // Flatten p into the plane passing through c perpendicular to the ray.
  // This gives the closest approach of the ray to the center.
  vec3 a = p - p_d * d;

  float aSquared = dot(a, a);

  // Closest approach is outside the sphere.
  if(aSquared > rSquared) {
    return vec3(-1.0);
  }

  // Calculate distance from plane where ray enters/exits the sphere.    
  float h = sqrt(rSquared - aSquared);

  // Calculate intersection point relative to sphere center.
  vec3 i = a - h * d;

  return c + i;
}

// Ray marching code
void step_ray(inout RayObject ray) {
  ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
  ray.side_dist += vec3(ray.mask) * ray.delta_dist;
  ray.map_pos += ivec3(vec3(ray.mask)) * ray.step;
}

// object_hit_distance() < 0 means that no object was hit
// this will return the point in world-space where the object
// was hit
vec3 object_hit_distance(WorldObject selected_object, RayObject ray) {
  // if we are checking this cube we definitely hit the cube objects
  if (selected_object.type == CUBE_WALL || selected_object.type == OPTICAL_OBJECT_CUBE) {
    float distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
    return vec3(ray.map_pos) - ray.dir * distance_traveled;
  }

  // if we hit a sphere type, we have to do additional checks
  if (selected_object.type == LIGHT_SOURCE) {
    return raySphereIntersectPos(ray.pos, selected_object.center, ray.dir, selected_object.radius);
  }

  // TODO: FIX
  if (selected_object.type == SQUARE_WALL || selected_object.type == OPTICAL_OBJECT_SQUARE_WALL) {
    vec3 a = rotate3dY(
        rotate3dX(
            vec3(
                -selected_object.width,
                0.0,
                selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 b = rotate3dY(
        rotate3dX(
            vec3(
                selected_object.width,
                0.0,
                selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 c = rotate3dY(
        rotate3dX(
            vec3(
                -selected_object.width,
                0.0,
                -selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 d = rotate3dY(
        rotate3dX(
            vec3(
                selected_object.width, 
                0.0,
                -selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 distance = quadIntersect(ray.pos, ray.dir, selected_object.center + a, selected_object.center + b, selected_object.center + c, selected_object.center + d);

    if (abs(distance.x) < selected_object.width) {
      // return distance;
    }

    // return -1.0;
  }

  if (selected_object.type == ROUND_WALL || selected_object.type == OPTICAL_OBJECT_ROUND_WALL) {
    vec3 a = rotate3dY(
        rotate3dX(
            vec3(
                -selected_object.width,
                0.0,
                selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 b = rotate3dY(
        rotate3dX(
            vec3(
                -selected_object.width,
                0.0,
                -selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 c = rotate3dY(
        rotate3dX(
            vec3(
                selected_object.width,
                0.0,
                selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 d = rotate3dY(
        rotate3dX(
            vec3(
                selected_object.width, 
                0.0,
                -selected_object.height
            ),
            selected_object.rotation.y
        ),
        selected_object.rotation.x
    );

    vec3 hit_pos_object_space = quadIntersect(ray.pos, ray.dir, selected_object.center, selected_object.center + b, selected_object.center + c, selected_object.center + d);
    float past_plane_product_ray = dot(ray.dir, hit_pos_object_space - ray.pos);

    if (length(hit_pos_object_space) < selected_object.radius * 2.0) {
      return hit_pos_object_space + selected_object.center;
    }

    return vec3(-1.0);
  }

  return vec3(-1.0);
}

// TODO find a better name, it doesn't only iterate, it tries to reach a goal
// the ray will simply iterate over the space in the direction it's facing trying to hit a 'solid' object
// Walls: will stop
// Models: will stop
// Mirrors: will bounce off the mirror and continue iterating
// Light source: will stop
// Optical Object: will multiply it's internal jones matrix and continue iterating
bool iterateRayInDirection(inout RayObject ray, ObjectGoal current_goal) {
  for (int i = 0; i < MAX_RAY_STEPS; i += 1) {
    step_ray(ray);

    uint hashed_value = hash(ray.map_pos + ivec3(100, 100, 100));
    uint original_index = hashed_value % uint(1000);
    uint current_index = buckets[original_index];

    float min_distance = 10000.0;
    bool found_at_least_one_object = false;
    uint closest_object_index = uint(0);

    // search the item in the "linked list" and save the closest one
    // a.k.a the first one we would hit
    while (current_index != U32_MAX) {
      if ((objects[current_index * uint(3)] == hashed_value) && (objects[(current_index * uint(3)) + uint(1)] != ray.object_hit)) {
        WorldObject object = get_object_at_index(objects[(current_index * uint(3)) + uint(1)]);
        vec3 pos_hit = object_hit_distance(object, ray);
        float curr_distance_traveled = length(pos_hit - ray.pos);

        bool is_valid_collision_target = (!current_goal.has_goal) || (object.type != LIGHT_SOURCE) || (objects[(current_index * uint(3)) + uint(1)] == current_goal.goal_index);

        if (all(greaterThan(pos_hit, vec3(-0.5))) && curr_distance_traveled < min_distance && is_valid_collision_target) {
          found_at_least_one_object = true;
          closest_object_index = current_index;
          min_distance = curr_distance_traveled;
        }
      }

      current_index = objects[(current_index * uint(3)) + uint(2)];
    }

    if (found_at_least_one_object) {
      ray.object_hit = objects[(closest_object_index * uint(3)) + uint(1)];
      ray.distance_traveled = min_distance;
      ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;

      WorldObject object_hit = get_object_at_index(ray.object_hit);

      // if we had a goal then check if we hit it
      if (current_goal.has_goal) {
        if (ray.object_hit == current_goal.goal_index) {
          float virtual_distance_traveled = ray.distance_traveled * cube_scaling_factor;

          ray.color.xyz *= 10.0 /(virtual_distance_traveled * virtual_distance_traveled);
          ray.color.xyz *= object_hit.color;
          return true;
        }

        // didn't hit whatever we were aiming for
        ray.color.xyz *= 0.05;

        return false;
      }

      ray.color.x = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(6)]);
      ray.color.y = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(7)]);
      ray.color.z = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(8)]);
      ray.color.a = 1.0;

      ray.ended_in_hit = true;

      return true;
    }

    if ((ray.map_pos.x >= 25 || ray.map_pos.x < 1) || 
        (ray.map_pos.y >= 25 || ray.map_pos.y < 1) ||
        (ray.map_pos.z >= 25 || ray.map_pos.z < 1)
    ) {
      ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
      ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;
      ray.object_hit = uint(0);
      ray.ended_in_hit = true;
      ray.color = vec4(vec3(ray.mask) * 0.2, 1.0) + vec4(0.05);

      float h = 2.0 + checker(ray.current_real_position);
      ray.color *= vec4(h, h, h, 1);

      return false;
    }
  }

  // should be unreachable
  return false;
}

void main() {
  vec2 screen_pos = ((gl_FragCoord.xy / viewport_dimensions) * 2.) - 1.;

  vec3 camera_dir = vec3(0.0, 0.0, 0.75);
  vec3 camera_plane_u = vec3(1.25, 0.0, 0.0);
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
    ray.object_hit = U32_MAX;

  ObjectGoal empty_goal;
    empty_goal.has_goal = false;

  // walls are not valid objects
  bool hit_valid_object = iterateRayInDirection(ray, empty_goal);

  WorldObject object_hit = get_object_at_index(ray.object_hit);

  if (light_sources_count != uint(0) && ray.object_hit == uint(0)) {
    ray.color *= 0.05;
  }

  object_found = vec4(float(ray.object_hit) / 255.0, 0.0, 0.0, 0.0);
  Polarization[166] lights_polarizations;
  uint light_sources_hit = uint(0);

  if (ray.ended_in_hit && object_hit.type != LIGHT_SOURCE) {
    for (uint light_source_index = uint(0); light_source_index < light_sources_count; light_source_index++) {
      WorldObject light_object = get_object_at_index(lights_definitions_indices[light_source_index]);

      ObjectGoal light_source_goal;
        light_source_goal.goal = light_object;
        light_source_goal.goal_index = lights_definitions_indices[light_source_index];
        light_source_goal.has_goal = true;

      bool ray_facing_light = true;

      // before we try reaching the light, we should check if we can
      // hit it without crossing the object we already hit
      // we won't be doing this for optical objects
      if (object_hit.type == ROUND_WALL) {
        vec3 wall_normal = rotate3dY(rotate3dX(vec3(0.0, 1.0, 0.0), object_hit.rotation.y), object_hit.rotation.x);
        float past_plane_product_light = dot(wall_normal, light_object.center - object_hit.center);
        float past_plane_product_ray = dot(wall_normal, ray.pos - object_hit.center);

        if ((past_plane_product_light > 0.0 && past_plane_product_ray < 0.0) || (past_plane_product_light < 0.0 && past_plane_product_ray > 0.0)) {
          out_color = vec4(0.0, 0.0, 0.0, 1.0);
          ray_facing_light = false;
          ray_facing_light = false;
        }

      } else if (object_hit.type == CUBE_WALL) {
        if (dot(vec3(ray.map_pos) - light_object.center, vec3(ray.mask) * -ray.dir) > 0.0) {
          out_color = vec4(0.0, 0.0, 0.0, 1.0);
          ray_facing_light = false;
          ray_facing_light = false;
        }
      }

      if (ray_facing_light) {
        RayObject bounced = ray;
          bounced.pos = ray.current_real_position;

          // point ray to light_source
          bounced.dir = normalize(light_object.center - bounced.current_real_position);

          bounced.map_pos = ivec3(bounced.pos);
          bounced.delta_dist = 1.0 / abs(bounced.dir);
          bounced.step = ivec3(sign(bounced.dir));
          bounced.side_dist = (sign(bounced.dir) * (vec3(bounced.map_pos) - bounced.pos) + (sign(bounced.dir) * 0.5) + 0.5) * bounced.delta_dist;
          bounced.mask = lessThanEqual(bounced.side_dist.xyz, min(bounced.side_dist.yzx, bounced.side_dist.zxy));
          bounced.ended_in_hit = false;

        if (iterateRayInDirection(bounced, light_source_goal)) {
          vec3 light_dir = vec3(0.0, 0.0, -1.0);
          light_dir = rotate3dX(light_dir, light_object.rotation.y);
          light_dir = rotate3dY(light_dir, light_object.rotation.x);
          light_dir = normalize(light_dir);

          // virtual distance
          float radius = computeDistance(light_object.center, light_object.center + light_dir, bounced.pos) * cube_scaling_factor;
          float z = length(light_object.center - ray.current_real_position) * cube_scaling_factor;
          float n = 1.0;

          Polarization polarization = light_object.polarization;
 
  //        if (ray_to_light.optical_objects_through_which_it_passed > 0) {
  //          polarization = cx_2x2_mat_x_cx_pol_mul(ray_to_light.optical_objects_found_product, polarization);
  //        }

          if (true) {
            // Gaussian beam definition
            // TODO: this should also be part of some light definition
            float wavelength = 1.0;
            float w0 = 1.0;
            float z_r = (PI * w0 * w0 * n) / wavelength;
            float w_z = w0 * sqrt(1.0 + pow(z / z_r, 2.0));
            float R_z = z * (1.0 + pow(z_r / z, 2.0));
            float gouy_z = atan(z / z_r);
            float k = (2.0 * PI * n) / wavelength;

            // Electric field definition
            // I just break it down into two parts for readability
            vec2 first_part_x_hat = cx_mul(polarization.Ex, vec2((w0 / w_z) * exp(-pow(radius, 2.0) / pow(w_z, 2.0)), 0));
            vec2 first_part_y_hat = cx_mul(polarization.Ey, vec2((w0 / w_z) * exp(-pow(radius, 2.0) / pow(w_z, 2.0)), 0));
            vec2 second_part = cx_exp(vec2(0.0, k * z + k * (pow(radius, 2.0) / (2.0 * R_z)) - gouy_z));

            if (dot(light_dir, ray.current_real_position - light_object.center) > 0.0) {
              polarization.Ex = cx_mul(first_part_x_hat, second_part) * 50.0;
              polarization.Ey = cx_mul(first_part_y_hat, second_part) * 50.0;
            } else {
              polarization.Ex = vec2(0.0);
              polarization.Ey = vec2(0.0);
            }
          } else if (true) {
            // spherical light source
            float A = 1.0;
            float r = length(ray.current_real_position - light_object.center) * cube_scaling_factor;

            polarization.Ex = cx_mul(polarization.Ex, cx_mul(vec2(A/r, 0.0), cx_exp(vec2(0.0, r)))) * 200.0;
            polarization.Ey = cx_mul(polarization.Ey, cx_mul(vec2(A/r, 0.0), cx_exp(vec2(0.0, r)))) * 200.0;
          } else {

          }

          lights_polarizations[light_source_index] = polarization;

          // we want to weigh the contribution of each light source to the color
          // before we do any fancy shmancy physics
          float current_light_intensity = pow(cx_abs(cx_add(polarization.Ex, polarization.Ey)), 2.0) / (2.0 * n);
          ray.color.xyz += bounced.color.xyz * current_light_intensity;
        }
      }
    }

    if (light_sources_count > uint(0)) {
      Polarization final_electric_field;
        final_electric_field.Ex = vec2(0, 0);
        final_electric_field.Ey = vec2(0, 0);

      // add up all electric fields
      for (uint i = uint(0); i < light_sources_count; i++) {
        final_electric_field.Ex = cx_add(lights_polarizations[i].Ex, final_electric_field.Ex);
        final_electric_field.Ey = cx_add(lights_polarizations[i].Ey, final_electric_field.Ey);
      }

      // color *= pow(cx_abs(final_electric_field.Ex), 2) + pow(cx_abs(final_electric_field.Ey), 2) + 0.2;
      vec2 Ex = final_electric_field.Ex;
      vec2 Ey = final_electric_field.Ey;
      // float result = pow(cx_abs(cx_add(Ex, Ey)), 2.0);
      float result = cx_abs(cx_add(cx_mul(Ex, cx_conj(Ex)), cx_mul(Ey, cx_conj(Ey))));
      result = max(background_light_min, result);

      ray.color *= result;
    }
  }

  out_color = ray.color;
}
