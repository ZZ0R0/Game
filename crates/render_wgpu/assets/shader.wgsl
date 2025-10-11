struct Camera { vp: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct WireframeData {
    enabled: f32,
    color: vec3<f32>,
}

@group(3) @binding(0) var<uniform> wireframe_data: WireframeData;

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
    var base_color = textureSample(tex, samp, in.uv);
    
    if wireframe_data.enabled > 0.5 {
        // In wireframe mode, make base texture transparent and use wireframe color
        let wireframe_alpha = 0.8;
        let base_alpha = 0.1; // Make original texture nearly transparent
        
        // Blend wireframe color with transparent base
        return vec4<f32>(
            mix(base_color.rgb * base_alpha, wireframe_data.color, wireframe_alpha),
            max(base_alpha, wireframe_alpha)
        );
    } else {
        // Normal rendering
        return base_color;
    }
}
