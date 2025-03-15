use crate::OPENGL_TO_WGPU_MATRIX;
use super::input::{InputHandler, InputType};

use cgmath::{EuclideanSpace, Vector3};
use winit::keyboard::KeyCode;

#[derive(Clone, Copy)]
pub struct Camera {
    pub scale: f32,
    pub position: [f32; 2],
    pub ang: [f32; 3],
    pub near: f32,
    pub far: f32,
    pub eye_position: Vector3<f32>,
}

const SCALE_MIN: f32 = 0.1;
const SCALE_MAX: f32 = 50.0;
const SCALE_SPEED: f32 = 1.0;
pub const ANGLES: [f32; 3] = [0.0, 0.0, 0.0];
const ANG_SPEED: f32 = 1.0;
const SPEED: f32 = 0.1;

impl Camera {
    pub fn build_view_projection_matrix(&mut self) -> cgmath::Matrix4<f32> {
        let rotation_x = cgmath::Matrix3::from_angle_x(cgmath::Deg(self.ang[0]));
        let rotation_y = cgmath::Matrix3::from_angle_y(cgmath::Deg(self.ang[1]));
        let rotation_z = cgmath::Matrix3::from_angle_z(cgmath::Deg(self.ang[2]));

        let mut camera_position = cgmath::Vector3::new(-1.0, 1.0, -1.0);
        camera_position = rotation_x * rotation_y * rotation_z * camera_position;
        self.eye_position = camera_position;
        let camera_position = cgmath::Point3::from_vec(camera_position);

        let look_direction = (0.0,0.0,0.0).into();
        let up_direction = cgmath::Vector3::unit_y();

        let view_mat = cgmath::Matrix4::look_at_rh(camera_position, look_direction, up_direction);

        let proj = cgmath::ortho(
            -4.0 / self.scale + self.position[0],
            4.0 / self.scale + self.position[0],
            -4.0 / self.scale + self.position[1],
            4.0 / self.scale + self.position[1],
            self.near,
            self.far,
        );

        


        return OPENGL_TO_WGPU_MATRIX * proj * view_mat;
    }

    pub fn controller(
        &mut self,
        input: &InputHandler,
        camera_uniform: &mut CameraUniform,
        queue: &mut wgpu::Queue,
        camera_buffer: &wgpu::Buffer,
    ) {
        if input.check_key( KeyCode::ArrowLeft, InputType::Held) {
            self.ang[1] = (self.ang[1] + ANG_SPEED + 360.0) % 360.0;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        } else if input.check_key( KeyCode::ArrowRight, InputType::Held) {
            self.ang[1] = (self.ang[1] - ANG_SPEED + 360.0) % 360.0;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        }
        if input.check_key( KeyCode::ArrowUp, InputType::Held) {
            let scale_step = self.scale / SCALE_MAX * SCALE_SPEED;
            self.scale = SCALE_MAX.min(self.scale + scale_step);
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        } else if input.check_key( KeyCode::ArrowDown, InputType::Held) {
            let scale_step = self.scale / SCALE_MAX * SCALE_SPEED;
            self.scale = SCALE_MIN.max(self.scale - scale_step);
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        }
        if input.check_key( KeyCode::KeyW, InputType::Held) {
            self.position[1] += SPEED / self.scale;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        } else if input.check_key( KeyCode::KeyS, InputType::Held) {
            self.position[1] -= SPEED / self.scale;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        }
        if input.check_key( KeyCode::KeyA, InputType::Held) {
            self.position[0] -= SPEED / self.scale;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        } else if input.check_key( KeyCode::KeyD, InputType::Held) {
            self.position[0] += SPEED / self.scale;
            camera_uniform.update_view_proj(self);
            queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]))
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    _padding: [f32; 2],
    pub relation: [f32; 2],
}

impl CameraUniform {
    pub fn new(width: f32, height: f32) -> Self {
        use cgmath::SquareMatrix;
        let relation = [
            if width > height { height / width } else { 1.0 },
            if width > height { 1.0 } else { width / height },
        ];
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            _padding: [0.0, 0.0],
            relation,
        }
    }

    pub fn update_view_proj(&mut self, camera: &mut Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
