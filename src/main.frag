precision mediump float;
in vec4 v_color;
out vec4 out_color;

void main() {
  vec2 viewport_dimensions = vec2(1000., 1000.);
  vec2 screen_pos = ((gl_FragCoord.xy / viewport_dimensions) * 2.) - 1.;

  vec3 camera_dir = vec3(0.0, 0.0, 1.0);
  vec3 camera_plane_u = vec3(1.0, 0.0, 0.0);
  vec3 camera_plane_v = vec3(0.0, 1.0, 0.0);

  out_color = vec4(gl_FragCoord.x / 1000., gl_FragCoord.y / 1000., 0., 1.);
}
