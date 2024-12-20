precision mediump float;

out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;
uniform sampler2D objects_found;
uniform vec2 viewport_dimensions;
uniform uint selected_object;
uniform float time;

bool is_approx(float in_val, float comp_val) {
  float epsilon = 0.0001;
  return (in_val > comp_val - epsilon) && (in_val < comp_val + epsilon);
}

void main() {
  vec2 screen_pos = (gl_FragCoord.xy) / viewport_dimensions;
  vec3 rgb_object_found = texture(objects_found, screen_pos).rgb;

  if ((selected_object != uint(0)) && (is_approx(float(selected_object), rgb_object_found.x * 255.0))) {
    // FragColor = vec4(texture(screenTexture, screen_pos).rgb * ((cos(time * 10.0) * 0.1) + 0.8), 1.);
    FragColor = vec4(normalize(texture(screenTexture, screen_pos).rgb), 1.);

  } else {
    FragColor = vec4(texture(screenTexture, screen_pos).rgb, 1.);
  }
} 
