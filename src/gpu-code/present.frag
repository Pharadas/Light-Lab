precision mediump float;

out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;
uniform sampler2D objects_found;
uniform vec2 viewport_dimensions;

void main() {
  vec2 texture_dimensions = vec2(250., 250.);

  // vec2 screen_pos = (gl_FragCoord.xy / viewport_dimensions) * texture_dimensions;
  vec2 screen_pos = (gl_FragCoord.xy) / viewport_dimensions;
  // FragColor = vec4(screen_pos, 1.0, 1.0);
  // vec3 col = texture(screenTexture, TexCoords).rgb;
  vec3 rgb_object_found = texture(objects_found, screen_pos).rgb;

  if (rgb_object_found == vec3(0., 0., 0.)) {
    FragColor = vec4(texture(screenTexture, screen_pos).rgb, 1.);

  } else {
    FragColor = vec4(rgb_object_found, 1.);
  }
} 
