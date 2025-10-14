// Space Engineers 3D Shader System

// Vertex input structure
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

// Camera uniform buffer
struct Camera {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

// Model matrix uniform
struct Model {
    matrix: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> model: Model;

// Texture and sampler
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

// Vertex shader - transforms 3D positions
@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_pos = model.matrix * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_pos.xyz;
    
    // Transform to clip space
    out.clip_position = camera.view_proj * world_pos;
    
    // Transform normal to world space
    out.world_normal = normalize((model.matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    
    out.tex_coords = vertex.tex_coords;
    
    return out;
}

// Fragment shader - basic lighting for Space Engineers blocks
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Basic lighting setup
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3)); // Sun direction
    let light_color = vec3<f32>(1.0, 0.95, 0.8); // Warm sunlight
    let ambient = vec3<f32>(0.1, 0.1, 0.15); // Space ambient
    
    // Calculate lighting
    let normal = normalize(in.world_normal);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse = diffuse_strength * light_color;
    
    // Apply lighting to texture
    let final_color = (ambient + diffuse) * tex_color.rgb;
    
    return vec4<f32>(final_color, tex_color.a);
}