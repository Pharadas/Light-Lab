precision mediump float;
precision mediump int;
in vec4 v_color;

uniform vec2 u_rotation;
uniform vec3 position; 
uniform vec2 viewport_dimensions;
uniform float time;
uniform uint light_sources_count;

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

float checker(vec3 p) {
  float t = 10.0;
  return step(0.0, sin(PI * p.x + PI/t)*sin(PI *p.y + PI/t)*sin(PI *p.z + PI/t));
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
    return ray.pos + (ray.dir * vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
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

    if (length(hit_pos_object_space) < selected_object.radius * 2.0) {
      return hit_pos_object_space + selected_object.center;
    }

    return vec3(-1.0);
  }

  return vec3(-1.0);
}

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
    while (current_index != U32_MAX) {
      if ((objects[current_index * uint(3)] == hashed_value) && (objects[(current_index * uint(3)) + uint(1)] != ray.object_hit)) {
        WorldObject object = get_object_at_index(objects[(current_index * uint(3)) + uint(1)]);
        vec3 pos_hit = object_hit_distance(object, ray);
        float curr_distance_traveled = length(pos_hit - ray.pos);

        if (all(greaterThan(pos_hit, vec3(-0.5))) && curr_distance_traveled < min_distance) {
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
          ray.color.xyz *= 50.0 / ray.distance_traveled;
          ray.color.xyz *= object_hit.color;
          return true;
        }

        // didn't hit whatever we were aiming for
        // ray.color.xyz *= 30.0 / ray.distance_traveled;
        // ray.color.xyz *= object_hit.color;
        ray.color.xyz *= 0.1;

        return false;
      }

      ray.color.x = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(6)]);
      ray.color.y = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(7)]);
      ray.color.z = uintBitsToFloat(objects_definitions[(ray.object_hit * OBJECT_SIZE) + uint(8)]);
      ray.color.a = 1.0;

      ray.pos = ray.dir * min_distance;

      ray.ended_in_hit = true;

      return true;
    }

    if ((ray.map_pos.x > 100 || ray.map_pos.x <= 1) || 
        (ray.map_pos.y > 100 || ray.map_pos.y <= 1) ||
        (ray.map_pos.z > 100 || ray.map_pos.z <= 1)
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
    ray.object_hit = U32_MAX;

  ObjectGoal empty_goal;
    empty_goal.has_goal = false;

  // walls are not valid objects
  bool hit_valid_object = iterateRayInDirection(ray, empty_goal);

  WorldObject object_hit = get_object_at_index(ray.object_hit);

  object_found = vec4(float(ray.object_hit) / 255.0, 0.0, 0.0, 0.0);

  if (ray.ended_in_hit && object_hit.type != LIGHT_SOURCE) {
    for (uint light_source_index = uint(0); light_source_index < light_sources_count; light_source_index++) {
      WorldObject light_object = get_object_at_index(lights_definitions_indices[light_source_index]);

      ObjectGoal light_source_goal;
        light_source_goal.goal = light_object;
        light_source_goal.goal_index = lights_definitions_indices[light_source_index];
        light_source_goal.has_goal = true;

      RayObject bounced = ray;
        bounced.pos = bounced.current_real_position;

        // point ray to light_source
        bounced.dir = normalize(light_object.center - bounced.current_real_position);

        bounced.map_pos = ivec3(bounced.pos);
        bounced.delta_dist = 1.0 / abs(bounced.dir);
        bounced.step = ivec3(sign(bounced.dir));
        bounced.side_dist = (sign(bounced.dir) * (vec3(bounced.map_pos) - bounced.pos) + (sign(bounced.dir) * 0.5) + 0.5) * bounced.delta_dist;
        bounced.mask = lessThanEqual(bounced.side_dist.xyz, min(bounced.side_dist.yzx, bounced.side_dist.zxy));
        bounced.ended_in_hit = false;

      iterateRayInDirection(bounced, light_source_goal);

      ray.color.xyz *= bounced.color.xyz;
      // ray.color.xyz = ray.dir;
    }
  }

  out_color = ray.color;
}
