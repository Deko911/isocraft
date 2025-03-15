use std::{io::BufReader, str::FromStr};

use wgpu::util::DeviceExt;

pub const SCALE: f32 = 1.0;

#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
    pub texture: BindTexture,
    pub position: [f32; 3],
    pub scale: f32,
}

impl Model {
    pub fn new(
        url_model: &str,
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        queue: &wgpu::Queue,
        label: &str,
        position: [f32; 3],
        scale: f32,
    ) -> Self {
        let mut mesh = Mesh::from_file(url_model, device);
        mesh.vertex.iter_mut().for_each(|x| {
            x.position = [
                x.position[0] * scale + position[0] / SCALE,
                x.position[1] * scale + position[1] / SCALE,
                x.position[2] * scale + position[2] / SCALE,
            ]
        });
        queue.write_buffer(&mesh.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertex));

        let texture = BindTexture::new(texture_bind_group_layout, bytes, device, queue, label);
        Self {
            mesh,
            texture,
            position,
            scale,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue){
        self.mesh.vertex.iter_mut().for_each(|x| {
            x.position = [
                x.position[0] * self.scale + self.position[0] / SCALE,
                x.position[1] * self.scale + self.position[1] / SCALE,
                x.position[2] * self.scale + self.position[2] / SCALE,
            ]
        });
        queue.write_buffer(&self.mesh.vertex_buffer, 0, bytemuck::cast_slice(&self.mesh.vertex));
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        render_pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &self.texture.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex: Vec<Vertex>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Mesh {
    pub fn new(vertex: Vec<Vertex>, indices: Vec<u16>, device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        Self {
            vertex,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    pub fn from_file(url_model: &str, device: &wgpu::Device) -> Self {
        use obj::*;
        use std::fs::File;

        let path = if !cfg!(debug_assertions) {
            let mut path =
                std::env::current_exe().expect("Error para conseguir la ruta del ejecutable");
            path.pop();
            path.push(url_model);
            path
        } else {
            std::path::PathBuf::from_str(url_model).expect("Error en la ruta")
        };

        let input = BufReader::new(File::open(path).expect("Ruta no encontrada"));
        let obj: Obj<TexturedVertex, u16> =
            obj::load_obj(input).expect("Falla al cargar el modelo");

        let vertex = obj
            .vertices
            .iter()
            .map(|x| super::model::Vertex::new(x.position, [x.texture[0], x.texture[1]]))
            .collect::<Vec<_>>();
        let indices = obj.indices;

        Mesh::new(vertex, indices, device)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BindTexture {
    pub diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,
}

impl BindTexture {
    pub fn new(
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Self {
        let texture = Texture::new(bytes, device, queue, label);
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        Self {
            diffuse_bind_group,
            diffuse_texture: texture,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    fn new(bytes: &[u8], device: &wgpu::Device, queue: &wgpu::Queue, label: &str) -> Self {
        let diffuse_image = image::load_from_memory(bytes).unwrap();
        let rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn desc() -> wgpu::BindGroupLayoutDescriptor<'static>{
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        }
    }

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    uv_coord: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], uv_coord: [f32; 2]) -> Self {
        Self { position, uv_coord }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    #[allow(dead_code)]
    pub fn rotate(&mut self, sinx: f32, cosx: f32, siny: f32, cosy: f32, sinz: f32, cosz: f32) {
        let pos_x = self.position[0];
        let pos_y = self.position[1];
        let pos_z = self.position[2];
        let x = pos_y * sinx * siny * cosz - pos_z * cosx * siny * cosz
            + pos_y * cosx * sinz
            + pos_z * sinx * sinz
            + pos_x * cosy * cosz;
        let y = pos_y * cosx * cosz + pos_z * sinx * cosz - pos_y * sinx * siny * sinz
            + pos_z * cosx * siny * sinz
            - pos_x * cosy * sinz;
        let z = pos_z * cosx * cosy - pos_y * sinx * cosy + pos_x * siny;
        self.position[0] = x;
        self.position[1] = y;
        self.position[2] = z;
    }
}
