// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    _padding: vec2<f32>,
    relation: vec2<f32>
}

struct UVs {
    uv_coords: vec2<f32>,
    uv_index: u32,
}

struct Uniforms {
    uvs: array<UVs, 12>,
    m_matrix: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) data: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) coords_uv: vec2<f32>,
    @location(2) shading: f32,
    @location(3) opacity: f32,
};

const face_shading: array<f32, 6> = array<f32, 6>(
    1.0, 0.5, //Top Bottom
    0.5, 0.8, //Right Left
    0.5, 0.8  //Front Back
);

const ao_values: array<f32, 4> = array<f32, 4>(
    0.1, 0.25, 0.5, 1.0
);

const SCALE: f32 = 32.0;

fn hash32(p: f32) -> vec3<f32>{
    var p3: vec3<f32>;
    p3 = fract(vec3<f32>(p * 21.2) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xxy + p3.yzz) * p3.zyx) + 0.05;
}

fn face_shading_const(index: u32) -> f32{
    switch index {
        case 0u: {
            return face_shading[0];
        }
        case 1u: {
            return face_shading[1];
        }
        case 2u: {
            return face_shading[2];
        }
        case 3u: {
            return face_shading[3];
        }
        case 4u: {
            return face_shading[4];
        }
        case 5u: {
            return face_shading[5];
        }
        default {
            return 0.0;
        }
    };
}

fn ao_const(index: u32) -> f32{
    switch index {
        case 0u: {
            return ao_values[0];
        }
        case 1u: {
            return ao_values[1];
        }
        case 2u: {
            return ao_values[2];
        }
        case 3u: {
            return ao_values[3];
        }
        default {
            return 0.0;
        }
    };
}

fn unpack(data: u32) -> array<u32, 7>{
    //a, b, c, d, e, f, g => x, y, z, voxel_id, face_id, shading_id, padding
    let bits = array<u32, 6>(6, 6, 8, 3, 2, 1);
    let masks = array<u32, 6>(63, 63, 255, 7, 3, 1);

    let fg_bit =     bits[4] + bits[5];
    let efg_bit =    bits[3] + fg_bit;
    let defg_bit =   bits[2] + efg_bit;
    let cdefg_bit =  bits[1] + defg_bit;
    let bcdefg_bit = bits[0] + cdefg_bit;

    let x = (data >> bcdefg_bit);
    let y = (data >> cdefg_bit) & masks[0];
    let z = (data >> defg_bit) & masks[1];
    let voxel_id = (data >> efg_bit) & masks[2];
    let face_id = (data >> fg_bit) & masks[3];
    let shading_id = (data >> bits[5]) & masks[4];
    let padding = data & masks[5];

    return array<u32, 7>(x, y, z, voxel_id, face_id, shading_id, padding);
}

const states: f32 = 8.0;

@vertex
fn vs_main(
    @builtin(vertex_index) vertexIndex: u32,
    model: VertexInput,
) -> VertexOutput {
    let data = unpack(model.data);
    let position = vec3<f32>(f32(data[0]) / SCALE, f32(data[1]) / SCALE, f32(data[2]) / SCALE);
    let voxel_id = data[3];
    let face_id = data[4];
    let shading_id = data[5];
    let select = bool(data[6]);
    var out: VertexOutput;
    let pos =  camera.view_proj * uniforms.m_matrix * vec4<f32>(position, 1.0);
    let color = vec3<f32>(hash32(f32(voxel_id)));
    let uv_index: u32 = vertexIndex % 6 + (face_id & 1) * 6;
    out.coords_uv = uniforms.uvs[uniforms.uvs[uv_index].uv_index].uv_coords;
    out.coords_uv.y = 1.0 - out.coords_uv.y;
    if face_id == 0{
        out.coords_uv = vec2<f32>(2.0/3.0 + out.coords_uv.x / 3.0, f32(voxel_id) / states + out.coords_uv.y / states);
    }else {
        out.coords_uv = vec2<f32>(1.0/3.0 + out.coords_uv.x / 3.0, f32(voxel_id) / states + out.coords_uv.y / states);
    }
    out.color = color;
    out.shading = face_shading_const(face_id) * ao_const(shading_id);
    out.opacity = 1.0;
    if select{
        out.shading *= 0.0;
    }
    out.clip_position = pos * vec4<f32>(camera.relation, 1.0, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

const gamma: vec3<f32> = vec3<f32>(2.2);

const inv_gamma: vec3<f32> = vec3<f32>(1 / gamma);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec3<f32>;
    color = textureSample(t_diffuse, s_diffuse, in.coords_uv).xyz;
    //color = pow(color, gamma);

    //color = in.color * color;
    color = color * in.shading;

    //color = pow(color, inv_gamma);

    if !(color.x == 0.0 && color.y == 0.0 && color.z == 0.0){
        return vec4<f32>(color, in.opacity);
    }
    return vec4<f32>(color, 1.0);
    

    //return vec4<f32>(in.coords_uv, 1.0, 0.5);
}