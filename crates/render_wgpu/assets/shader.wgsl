struct Camera { vp: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VSIn {
  @location(0) pos: vec3<f32>,
  @location(1) uv:  vec2<f32>,
}

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VSIn) -> VSOut {
  var out: VSOut;
  out.pos = camera.vp * vec4<f32>(in.pos, 1.0);
  out.uv = in.uv;
  return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, in.uv);
}
