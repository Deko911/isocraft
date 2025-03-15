pub mod chunk;

use cgmath::Matrix4;
use chunk::*;
use rand::prelude::*;

pub const WORLD_W: u32 = 5;
pub const WORLD_H: u32 = 2;
pub const WORLD_D: u32 = 5;
pub const WORLD_AREA: u32 = WORLD_W * WORLD_D;
pub const WORLD_VOL: u32 = WORLD_AREA * WORLD_H;

pub struct World {
    pub chunks: Vec<ChunkMesh>,
    pub voxels: Vec<Vec<u8>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: vec![],
            voxels: vec![],
        }
    }

    pub fn update(&mut self) {}

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        render_pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
        camera_m: Matrix4<f32>,
        relation: [f32; 2],
    ) {
        self.chunks
            .iter()
            /* .filter(|c| {
                let [cx, cy, cz] = c.position;
                let points = [
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 1.0],
                    [1.0, 1.0, 0.0],
                    [1.0, 1.0, 1.0],
                ];
                let position_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::from([
                    cx - WORLD_W as f32 / 2.0,
                    cy - WORLD_H as f32,
                    cz - WORLD_D as f32 / 2.0,
                ]));
                for i in points {
                    let projec_point = camera_m * position_matrix * Vector3::from(i).extend(1.0);
                    if  projec_point.x.abs() / relation[0] <= 1.0 || projec_point.y.abs() / relation[1] <= 1.0 {
                        return true;
                    }
                }
                false
            }) */
            .for_each(|x| x.render(render_pass, render_pipeline, camera_bind_group))
    }

    pub fn build_chunk(
        &mut self,
        device: &wgpu::Device,
        bytes: &[u8],
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        queue: &wgpu::Queue,
    ) {
        let seed: u32 = random();
        for y in 0..WORLD_H {
            for z in 0..WORLD_D {
                for x in 0..WORLD_W {
                    let chunk_voxels = ChunkMesh::voxels([x as f32, y as f32, z as f32], seed);
                    self.voxels.push(chunk_voxels);
                }
            }
        }
        for y in 0..WORLD_H {
            for z in 0..WORLD_D {
                for x in 0..WORLD_W {
                    let chunk = ChunkMesh::new(
                        device,
                        bytes,
                        texture_bind_group_layout,
                        queue,
                        [x as f32, y as f32, z as f32],
                        &self.voxels,
                        seed
                    );
                    self.chunks.push(chunk);
                }
            }
        }
    }
}
