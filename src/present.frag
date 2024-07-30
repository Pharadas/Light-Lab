precision mediump float;

out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;
uniform vec2 viewport_dimensions;

void main() {
  vec2 texture_dimensions = vec2(250., 250.);

  // vec2 screen_pos = (gl_FragCoord.xy / viewport_dimensions) * texture_dimensions;
  vec2 screen_pos = (gl_FragCoord.xy - vec2(-100., -50.0)) / viewport_dimensions;
  // FragColor = vec4(screen_pos, 1.0, 1.0);
  // vec3 col = texture(screenTexture, TexCoords).rgb;
  FragColor = vec4(texture(screenTexture, screen_pos).rgb, 1.);
} 
