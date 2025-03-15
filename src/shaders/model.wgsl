// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    _padding: vec2<f32>,
    relation: vec2<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coords_uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords_uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.coords_uv = model.coords_uv;
    let pos =  camera.view_proj * vec4<f32>(model.position, 1.0);
    out.clip_position = pos * vec4<f32>(camera.relation, 1.0, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.coords_uv);
}