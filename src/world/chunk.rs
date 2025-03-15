use noise::{NoiseFn, Simplex};
use wgpu::util::DeviceExt;

use crate::utils::model::BindTexture;

use super::{WORLD_D, WORLD_H, WORLD_W};

pub const CHUNK_SIZE: u8 = 32;
#[allow(dead_code)]
const CHUNK_SIZE_H: u8 = CHUNK_SIZE / 2;
pub const CHUNK_AREA: usize = CHUNK_SIZE as usize * CHUNK_SIZE as usize;
const CHUNK_VOL: usize = CHUNK_AREA * CHUNK_SIZE as usize;
const PERLIN_SCALE: f64 = 0.5;
pub const MAX_HEIGHT: f64 = 64.0;

#[derive(Debug)]
pub struct Chunk {
    pub voxels: Vec<u8>,
    is_empty: bool,
}

impl Chunk {
    fn new() -> Self {
        Self {
            voxels: vec![0; CHUNK_VOL],
            is_empty: false,
        }
    }

    fn build_voxels(&mut self, position: [f32; 3], seed: u32) {
        let perlin = Simplex::new(seed);
        let [cx, cy, cz] = [
            position[0] * CHUNK_SIZE as f32,
            position[1] * CHUNK_SIZE as f32,
            position[2] * CHUNK_SIZE as f32,
        ];

        let mut is_empty = true;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let wx = cx + x as f32;
                let wz = cz + z as f32;
                let point = [
                    wx as f64 / CHUNK_SIZE as f64 * PERLIN_SCALE,
                    wz as f64 / CHUNK_SIZE as f64 * PERLIN_SCALE,
                ];
                let world_height = (perlin.get(point) + 1.0) * MAX_HEIGHT / 2.0;
                let local_height = (CHUNK_SIZE as f64).min(world_height - cy as f64) as usize;
                for y in 0..local_height {
                    if y as f32 + cy == world_height.floor() as f32 - 1.0{
                        self.voxels[x as usize
                            + CHUNK_SIZE as usize * z as usize
                            + CHUNK_AREA as usize * y as usize] = 2;
                    } else {
                        self.voxels[x as usize
                            + CHUNK_SIZE as usize * z as usize
                            + CHUNK_AREA as usize * y as usize] = 3;
                    }

                    is_empty = false;
                }
            }
        }
        if is_empty {
            self.is_empty = true;
        }
    }

    fn get_ao(
        local_position: [i32; 3],
        global_position: [f32; 3],
        world_voxels: &Vec<Vec<u8>>,
        plane: Plane,
    ) -> [u8; 4] {
        let [x, y, z] = local_position;
        let [wx, wy, wz] = global_position;

        let ao;
        match plane {
            Plane::X => {
                let a = Chunk::voxel_is_void([x, y, z - 1], [wx, wy, wz - 1.0], world_voxels) as u8;
                let b =
                    Chunk::voxel_is_void([x, y - 1, z - 1], [wx, wy - 1.0, wz - 1.0], world_voxels)
                        as u8;
                let c = Chunk::voxel_is_void([x, y - 1, z], [wx, wy - 1.0, wz], world_voxels) as u8;
                let d =
                    Chunk::voxel_is_void([x, y - 1, z + 1], [wx, wy - 1.0, wz + 1.0], world_voxels)
                        as u8;
                let e = Chunk::voxel_is_void([x, y, z + 1], [wx, wy, wz + 1.0], world_voxels) as u8;
                let f =
                    Chunk::voxel_is_void([x, y + 1, z + 1], [wx, wy + 1.0, wz + 1.0], world_voxels)
                        as u8;
                let g = Chunk::voxel_is_void([x, y + 1, z], [wx, wy + 1.0, wz], world_voxels) as u8;
                let h =
                    Chunk::voxel_is_void([x, y + 1, z - 1], [wx, wy + 1.0, wz - 1.0], world_voxels)
                        as u8;
                ao = [a + b + c, g + h + a, e + f + g, c + d + e];
            }
            Plane::Y => {
                let a = Chunk::voxel_is_void([x, y, z - 1], [wx, wy, wz - 1.0], world_voxels) as u8;
                let b =
                    Chunk::voxel_is_void([x - 1, y, z - 1], [wx - 1.0, wy, wz - 1.0], world_voxels)
                        as u8;
                let c = Chunk::voxel_is_void([x - 1, y, z], [wx - 1.0, wy, wz], world_voxels) as u8;
                let d =
                    Chunk::voxel_is_void([x - 1, y, z + 1], [wx - 1.0, wy, wz + 1.0], world_voxels)
                        as u8;
                let e = Chunk::voxel_is_void([x, y, z + 1], [wx, wy, wz + 1.0], world_voxels) as u8;
                let f =
                    Chunk::voxel_is_void([x + 1, y, z + 1], [wx + 1.0, wy, wz + 1.0], world_voxels)
                        as u8;
                let g = Chunk::voxel_is_void([x + 1, y, z], [wx + 1.0, wy, wz], world_voxels) as u8;
                let h =
                    Chunk::voxel_is_void([x + 1, y, z - 1], [wx + 1.0, wy, wz - 1.0], world_voxels)
                        as u8;
                ao = [a + b + c, g + h + a, e + f + g, c + d + e];
            }
            Plane::Z => {
                let a = Chunk::voxel_is_void([x - 1, y, z], [wx - 1.0, wy, wz], world_voxels) as u8;
                let b =
                    Chunk::voxel_is_void([x - 1, y - 1, z], [wx - 1.0, wy - 1.0, wz], world_voxels)
                        as u8;
                let c = Chunk::voxel_is_void([x, y - 1, z], [wx, wy - 1.0, wz], world_voxels) as u8;
                let d =
                    Chunk::voxel_is_void([x + 1, y - 1, z], [wx + 1.0, wy - 1.0, wz], world_voxels)
                        as u8;
                let e = Chunk::voxel_is_void([x + 1, y, z], [wx + 1.0, wy, wz], world_voxels) as u8;
                let f =
                    Chunk::voxel_is_void([x + 1, y + 1, z], [wx + 1.0, wy + 1.0, wz], world_voxels)
                        as u8;
                let g = Chunk::voxel_is_void([x, y + 1, z], [wx, wy + 1.0, wz], world_voxels) as u8;
                let h =
                    Chunk::voxel_is_void([x - 1, y + 1, z], [wx - 1.0, wy + 1.0, wz], world_voxels)
                        as u8;
                ao = [a + b + c, g + h + a, e + f + g, c + d + e];
            }
        }
        ao
    }

    fn get_index(position: [f32; 3]) -> isize {
        let [wx, wy, wz] = position;
        let cx = (wx / CHUNK_SIZE as f32).floor() as i32;
        let cy = (wy / CHUNK_SIZE as f32).floor() as i32;
        let cz = (wz / CHUNK_SIZE as f32).floor() as i32;

        use super::*;

        if !(0 <= cx
            && cx < WORLD_W as i32
            && 0 <= cy
            && cy < WORLD_H as i32
            && 0 <= cz
            && cz < WORLD_D as i32)
        {
            return -1;
        }

        (cx as f32 + WORLD_W as f32 * cz as f32 + WORLD_AREA as f32 * cy as f32) as isize
    }

    fn voxel_is_void(
        local_position: [i32; 3],
        global_position: [f32; 3],
        world_voxels: &Vec<Vec<u8>>,
    ) -> bool {
        let index = Chunk::get_index(global_position);

        if index == -1 {
            return true;
        }
        let chunk_voxels = &world_voxels[index as usize];

        let [x, y, z] = local_position;

        let voxel_index = ((x as f32 + CHUNK_SIZE as f32) % CHUNK_SIZE as f32
            + (z as f32 + CHUNK_SIZE as f32) % CHUNK_SIZE as f32 * CHUNK_SIZE as f32
            + (y as f32 + CHUNK_SIZE as f32) % CHUNK_SIZE as f32 * CHUNK_AREA as f32)
            as usize;

        if chunk_voxels[voxel_index] > 0 {
            return false;
        }

        true
    }

    fn add_vertex(
        chunk_voxels: &mut Vec<ChunkVertexPacked>,
        vertex: &[[u8; 7]],
        index: &mut usize,
    ) {
        for vertex in vertex {
            let [x, y, z, voxel_id, face_id, shading_id, select] = *vertex;
            chunk_voxels[*index] =
                ChunkVertexPacked::pack_data(x, y, z, voxel_id, face_id, shading_id, select);
            *index += 1;
        }
    }

    fn build_mesh(
        &mut self,
        position: [f32; 3],
        world_voxels: &Vec<Vec<u8>>,
        sel: Option<usize>,
    ) -> Vec<ChunkVertexPacked> {
        let mut vertex_data: Vec<ChunkVertexPacked> =
            vec![ChunkVertexPacked::pack_data(0, 0, 0, 0, 0, 0, 0); CHUNK_VOL * 15];
        let mut index = 0;

        if self.is_empty {
            return vec![];
        }

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let voxel_id = self.voxels[x as usize
                        + CHUNK_SIZE as usize * z as usize
                        + CHUNK_AREA as usize * y as usize];

                    if voxel_id == 0 {
                        continue;
                    }

                    {
                        let mut select = 0;
                        if let Some(selec) = sel {
                            if index == selec {
                                select = 1;
                            }
                        }

                        let [cx, cy, cz] = position;
                        let wx = x as f32 + cx * CHUNK_SIZE as f32;
                        let wy = y as f32 + cy * CHUNK_SIZE as f32;
                        let wz = z as f32 + cz * CHUNK_SIZE as f32;

                        let (x, y, z) = (x as i32, y as i32, z as i32);
                        if Chunk::voxel_is_void([x, y + 1, z], [wx, wy + 1.0, wz], world_voxels) {
                            let ao = Chunk::get_ao(
                                [x, y + 1, z],
                                [wx, wy + 1.0, wz],
                                world_voxels,
                                Plane::Y,
                            );
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x, y + 1, z, voxel_id, 0, ao[0], select];
                            let v1 = [x + 1, y + 1, z, voxel_id, 0, ao[1], select];
                            let v2 = [x + 1, y + 1, z + 1, voxel_id, 0, ao[2], select];
                            let v3 = [x, y + 1, z + 1, voxel_id, 0, ao[3], select];

                            Chunk::add_vertex(
                                &mut vertex_data,
                                &[v1, v0, v3, v1, v3, v2],
                                &mut index,
                            );
                        }

                        /*  if Chunk::voxel_is_void([x, y - 1, z], [wx, wy - 1.0, wz], world_voxels) {
                            let ao = Chunk::get_ao([x, y - 1, z], [wx, wy - 1.0, wz], world_voxels, Plane::Y);
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x    , y, z    , voxel_id, 1, ao[0], select];
                            let v1 = [x + 1, y, z    , voxel_id, 1, ao[1], select];
                            let v2 = [x + 1, y, z + 1, voxel_id, 1, ao[2], select];
                            let v3 = [x    , y, z + 1, voxel_id, 1, ao[3], select];

                            Chunk::add_vertex(&mut vertex_data, &[v3, v0, v2, v0, v1, v2], &mut index);
                        } */

                        //Front
                        if Chunk::voxel_is_void([x + 1, y, z], [wx + 1.0, wy, wz], world_voxels) {
                            let ao = Chunk::get_ao(
                                [x + 1, y, z],
                                [wx + 1.0, wy, wz],
                                world_voxels,
                                Plane::X,
                            );
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x + 1, y, z, voxel_id, 2, ao[0], select];
                            let v1 = [x + 1, y + 1, z, voxel_id, 2, ao[1], select];
                            let v2 = [x + 1, y + 1, z + 1, voxel_id, 2, ao[2], select];
                            let v3 = [x + 1, y, z + 1, voxel_id, 2, ao[3], select];

                            Chunk::add_vertex(
                                &mut vertex_data,
                                &[v2, v3, v0, v2, v0, v1],
                                &mut index,
                            );
                        }

                        //Back
                        if Chunk::voxel_is_void([x - 1, y, z], [wx - 1.0, wy, wz], world_voxels) {
                            let ao = Chunk::get_ao(
                                [x - 1, y, z],
                                [wx - 1.0, wy, wz],
                                world_voxels,
                                Plane::X,
                            );
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x, y, z, voxel_id, 3, ao[0], select];
                            let v1 = [x, y + 1, z, voxel_id, 3, ao[1], select];
                            let v2 = [x, y + 1, z + 1, voxel_id, 3, ao[2], select];
                            let v3 = [x, y, z + 1, voxel_id, 3, ao[3], select];

                            Chunk::add_vertex(
                                &mut vertex_data,
                                &[v2, v0, v3, v2, v1, v0],
                                &mut index,
                            );
                        }

                        if Chunk::voxel_is_void([x, y, z + 1], [wx, wy, wz + 1.0], world_voxels) {
                            let ao = Chunk::get_ao(
                                [x, y, z + 1],
                                [wx, wy, wz + 1.0],
                                world_voxels,
                                Plane::Z,
                            );
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x, y, z + 1, voxel_id, 4, ao[0], select];
                            let v1 = [x, y + 1, z + 1, voxel_id, 4, ao[1], select];
                            let v2 = [x + 1, y + 1, z + 1, voxel_id, 4, ao[2], select];
                            let v3 = [x + 1, y, z + 1, voxel_id, 4, ao[3], select];

                            Chunk::add_vertex(
                                &mut vertex_data,
                                &[v1, v0, v3, v1, v3, v2],
                                &mut index,
                            );
                        }

                        if Chunk::voxel_is_void([x, y, z - 1], [wx, wy, wz - 1.0], world_voxels) {
                            let ao = Chunk::get_ao(
                                [x, y, z - 1],
                                [wx, wy, wz - 1.0],
                                world_voxels,
                                Plane::Z,
                            );
                            let (x, y, z) = (x as u8, y as u8, z as u8);
                            let v0 = [x, y, z, voxel_id, 5, ao[0], select];
                            let v1 = [x, y + 1, z, voxel_id, 5, ao[1], select];
                            let v2 = [x + 1, y + 1, z, voxel_id, 5, ao[2], select];
                            let v3 = [x + 1, y, z, voxel_id, 5, ao[3], select];

                            Chunk::add_vertex(
                                &mut vertex_data,
                                &[v1, v3, v0, v1, v2, v3],
                                &mut index,
                            );
                        }
                    }
                }
            }
        }
        vertex_data
    }
}

#[derive(Debug)]
pub struct ChunkMesh {
    #[allow(dead_code)]
    pub chunk: Chunk,
    vertex_buffer: wgpu::Buffer,
    texture: BindTexture,
    mesh_size: u32,
    pub position: [f32; 3],
    chunk_bind_group: wgpu::BindGroup,
}

impl ChunkMesh {
    pub fn voxels(position: [f32; 3], seed: u32) -> Vec<u8> {
        let mut voxels = Chunk::new();
        voxels.build_voxels(position, seed);
        voxels.voxels
    }

    pub fn new(
        device: &wgpu::Device,
        bytes: &[u8],
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        queue: &wgpu::Queue,
        position: [f32; 3],
        world_voxels: &Vec<Vec<u8>>,
        seed: u32
    ) -> ChunkMesh {
        ChunkVertexPacked::pack_data(1, 1, 1, 1, 1, 1, 0);
        let mut chunk = Chunk::new();
        chunk.build_voxels(position, seed);
        let vertex = chunk.build_mesh(position, world_voxels, None);
        let texture = BindTexture::new(texture_bind_group_layout, bytes, device, queue, "Terrain");
        //let mut vertex: Vec<ChunkVertex> = vec![ChunkVertex::new(0, 0, 0, 0, 0, 0); mesh.len() + (CHUNK_VOL * 30 - mesh.len()) / 2];
        let position_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::from([
            position[0] - WORLD_W as f32 / 2.0,
            position[1] - WORLD_H as f32,
            position[2] - WORLD_D as f32 / 2.0,
        ]));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let chunk_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[Uniforms::new(position_matrix.into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let chunk_bind_group_layout = device.create_bind_group_layout(&Uniforms::desc());

        let chunk_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk's uniforms bind group"),
            layout: &chunk_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: chunk_buffer.as_entire_binding(),
            }],
        });

        Self {
            chunk,
            vertex_buffer,
            texture,
            mesh_size: vertex.len() as u32,
            position,
            chunk_bind_group,
        }
    }

    pub fn reflesh(
        &mut self,
        queue: &wgpu::Queue,
        world_voxels: &Vec<Vec<u8>>,
        select: Option<usize>,
    ) {
        let vertex = self.chunk.build_mesh(self.position, world_voxels, select);

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex))
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        render_pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        if !self.chunk.is_empty {
            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, &self.texture.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.chunk_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.mesh_size, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertexPacked {
    data: u32,
}

impl ChunkVertexPacked {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ChunkVertexPacked>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }
    fn pack_data(
        x: u8,
        y: u8,
        z: u8,
        voxel_id: u8,
        face_id: u8,
        shading_id: u8,
        select: u8,
    ) -> Self {
        // x: 6 bit, y: 6 bit, z: 6 bit, voxel_id: 8 bit, face_id: 3bit, ao_id: 2 bit, select: 1 bit
        let (a, b, c, d, e, f, g) = (x, y, z, voxel_id, face_id, shading_id, select);
        let (a, b, c, d, e, f, g) = (
            a as u32, b as u32, c as u32, d as u32, e as u32, f as u32, g as u32,
        );
        let (b_bit, c_bit, d_bit, e_bit, f_bit, g_bit) = (6, 6, 8, 3, 2, 1);
        let fg_bit = f_bit + g_bit;
        let efg_bit = e_bit + fg_bit;
        let defg_bit = d_bit + efg_bit;
        let cdefg_bit = c_bit + defg_bit;
        let bcdefg_bit = b_bit + cdefg_bit;

        let packed_data: u32 = a << bcdefg_bit
            | b << cdefg_bit
            | c << defg_bit
            | d << efg_bit
            | e << fg_bit
            | f << g_bit
            | g;

        Self { data: packed_data }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UvUniform {
    uv_coord: [f32; 2],
    uv_index: u32,
    _padding: u32,
}
impl UvUniform {
    fn new(uv_coord: [f32; 2], uv_index: u32) -> Self {
        Self {
            uv_coord,
            uv_index,
            _padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    uv_uniform: [UvUniform; 12],
    p_matrix: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new(p_matrix: [[f32; 4]; 4]) -> Self {
        let uv_uniform = [
            UvUniform::new([0.0, 0.0], 1),
            UvUniform::new([0.0, 1.0], 0),
            UvUniform::new([1.0, 0.0], 2),
            UvUniform::new([1.0, 1.0], 1),
            UvUniform::new([0.0, 0.0], 2),
            UvUniform::new([0.0, 0.0], 3),
            UvUniform::new([0.0, 0.0], 3),
            UvUniform::new([0.0, 0.0], 0),
            UvUniform::new([0.0, 0.0], 2),
            UvUniform::new([0.0, 0.0], 3),
            UvUniform::new([0.0, 0.0], 1),
            UvUniform::new([0.0, 1.0], 0),
        ];
        Self {
            uv_uniform,
            p_matrix,
        }
    }
    pub fn desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Chunk's UvUniforms bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }
}

enum Plane {
    X,
    Y,
    Z,
}
