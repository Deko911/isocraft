////////////////////////////////////////////////////////////////////
use crate::world::{
    chunk::{CHUNK_AREA, CHUNK_SIZE},
    World, WORLD_AREA, WORLD_D, WORLD_H, WORLD_W,
};
/// IMPORTANTE!!!!!!!                                             //  
/// EL PASO DE LAS VARIABLES ax, ay y az INFLUYEN EN LA PRECISION //
/// DEL SISTEMA, HACER QUE EL JUGADOR PUEDA ESCOGERLA             //
////////////////////////////////////////////////////////////////////

use super::camera;
use cgmath::{InnerSpace, Vector3, Vector4};
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Clone, Copy)]
pub struct VoxelHandler {
    pub voxel_index: Option<usize>,
    pub chunk_index: Option<usize>,
    pub last_position: Option<(usize, usize)>,
    pub last_state: Option<u8>,
    pub voxel_local_pos: Option<[f32; 3]>,
    pub voxel_world_pos: Option<[f32; 3]>,
    direction: Option<Direction>,
    pub last_world: Option<[f32; 3]>,
}

impl VoxelHandler {
    pub fn new() -> Self {
        Self {
            voxel_index: None,
            chunk_index: None,
            last_position: None,
            last_state: None,
            voxel_local_pos: None,
            voxel_world_pos: None,
            direction: None,
            last_world: None,
        }
    }

    pub fn update(
        &mut self,
        camera: &mut camera::Camera,
        mouse_pos: PhysicalPosition<f32>,
        size: PhysicalSize<u32>,
        relation: [f32; 2],
        world: &World,
    ) {
        //Constantes en la ejecucion de la funcion
        let c_matrix = camera.build_view_projection_matrix();
        let mut linex: cgmath::Vector4<f32> = cgmath::Vector4::unit_x() / CHUNK_SIZE as f32;
        let mut linez: cgmath::Vector4<f32> = cgmath::Vector4::unit_z() / CHUNK_SIZE as f32;
        linex = c_matrix * linex;
        linez = c_matrix * linez;
        let magx = (linex.x.powi(2) + linex.y.powi(2)).sqrt() / 2.0;
        let magz = (linez.x.powi(2) + linez.y.powi(2)).sqrt() / 2.0;
        let mx = if linex.x < 0.001 && linex.x > -0.001 {
            0.0
        } else {
            linex.y / linex.x
        };
        let mz = if linez.x < 0.001 && linez.x > -0.001 {
            0.0
        } else {
            linez.y / linez.x
        };
        ///////////////////////////////////////////////////////////////////////////

        let mut x = (mouse_pos.x / size.width as f32 - 0.5) / relation[0]
            + camera.position[0] * camera.scale / 8.0;

        let y = (0.5 - mouse_pos.y / size.height as f32) / relation[1]
            + camera.position[1] * camera.scale / 8.0;
        let z: f32;
        let bx = y - mx * x;
        let mut px = bx / (mz - mx);
        px = if px.is_infinite() { 0.0 } else { px };
        let py = mx * px + bx;
        if camera.ang[1] == 45.0 || camera.ang[1] == 225.0 {
            z = x * camera.ang[1].to_radians().sin().signum();
        } else if camera.ang[1] == 135.0 || camera.ang[1] == 315.0 {
            z = y * camera.ang[1].to_radians().cos().signum();
        } else {
            if camera.ang[1] > 45.0 && camera.ang[1] < 225.0 {
                z = (px.powf(2.0) + py.powf(2.0)).sqrt() * -py.signum();
            } else {
                z = (px.powf(2.0) + py.powf(2.0)).sqrt() * py.signum();
            }
        }
        if camera.ang[1] == 45.0 || camera.ang[1] == 225.0 {
            x = y * (y - py).signum();
        } else if camera.ang[1] == 135.0 || camera.ang[1] == 315.0 {
            x = x * camera.ang[1].to_radians().sin().signum();
        } else {
            if camera.ang[1] > 45.0 && camera.ang[1] < 225.0 {
                x = ((px - x).powf(2.0) + (py - y).powf(2.0)).sqrt() * (x - px).signum();
            } else {
                x = ((px - x).powf(2.0) + (py - y).powf(2.0)).sqrt() * (px - x).signum();
            }
        }

        let (mut x, mut y, mut z) = (
            x / magx + WORLD_W as f32 / 2.0 * CHUNK_SIZE as f32,
            WORLD_H as f32 * CHUNK_SIZE as f32,
            z / magz + WORLD_D as f32 / 2.0 * CHUNK_SIZE as f32,
        );

        let mag = camera.eye_position.magnitude();
        let d = -camera.eye_position / mag;

        let mut ay = 0.0;
        let (cx, cy, cz) = camera.eye_position.into();
        loop {
            let (px, py, pz): (f32, f32, f32);
            py = ay + y;
            px = cx + d.x / d.y * (ay - cy) + x;
            pz = cz + d.z / d.y * (ay - cy) + z;
            if !(px < 0.0
                || px >= WORLD_W as f32 * CHUNK_SIZE as f32
                || py < 0.0
                || py >= WORLD_H as f32 * CHUNK_SIZE as f32
                || pz < 0.0
                || pz >= WORLD_D as f32 * CHUNK_SIZE as f32)
            {
                let (voxel_index, chunk_index) = get_voxel(px, py, pz);
                if world.voxels[chunk_index][voxel_index] != 0 {
                    [x, y, z] = [px, py, pz];
                    break;
                }
            }

            if (WORLD_H * CHUNK_SIZE as u32) as f32 + ay < 0.0 {
                return;
            }
            ay -= 0.05;
        }

        let direction = self.get_direction(x, y, z);

        /* let [px, py, pz] = [
            Vector3::new(x.floor() + 0.5, y.floor() + 0.5, z.floor()      ),
            Vector3::new(x.floor() + 0.5, y.floor() + 1.0, z.floor() + 0.5),
            Vector3::new(x.floor()      , y.floor() + 0.5, z.floor() + 0.5),
        ];

        let point = Vector3::new(x, y, z);

        let [dx, dy, dz] = [point - px, point - py, point - pz];

        let [dx, dy, dz] = [dx.magnitude(), dy.magnitude(), dz.magnitude()];
        let min = dx.min(dy.min(dz)) */
        self.direction = Some(direction);

        self.voxel_world_pos = Some([x, y, z]);
        let [cx, cy, cz] = [
            x as u32 / CHUNK_SIZE as u32,
            y as u32 / CHUNK_SIZE as u32,
            z as u32 / CHUNK_SIZE as u32,
        ];
        let [x, y, z] = [
            x - cx as f32 * CHUNK_SIZE as f32,
            y - cy as f32 * CHUNK_SIZE as f32,
            z - cz as f32 * CHUNK_SIZE as f32,
        ];
        let local_pos = [x, y, z];
        self.voxel_local_pos = Some(local_pos);
        self.voxel_index =
            Some(x as usize + CHUNK_SIZE as usize * z as usize + CHUNK_AREA as usize * y as usize);
        self.chunk_index =
            Some(cx as usize + WORLD_D as usize * cz as usize + WORLD_AREA as usize * cy as usize);
    }

    fn get_direction(&self, x: f32, y: f32, z: f32) -> Direction{
        let points = [
            (Vector3::new(x.floor() + 0.5, y.floor() + 1.0, z.floor() + 0.5), Direction::Y(1.0)),
            (Vector3::new(x.floor() + 0.5, y.floor() + 0.5, z.floor()      ), Direction::Z(-1.0)),
            (Vector3::new(x.floor() + 0.5, y.floor() + 0.5, z.floor() + 1.0), Direction::Z(1.0)),
            (Vector3::new(x.floor()      , y.floor() + 0.5, z.floor() + 0.5), Direction::X(-1.0)),
            (Vector3::new(x.floor() + 1.0, y.floor() + 0.5, z.floor() + 0.5), Direction::X(1.0))
        ];

        let point = Vector3::new(x, y, z);

        let mut min = f32::INFINITY;
        let mut direction = Direction::Y(1.0);

        for (v, dir) in points{
            let d = point - v;
            if d.magnitude() < min{
                min = d.magnitude();
                direction = dir;
            }
        }
        direction
    }

    pub fn change_voxel(&mut self, world: &mut World, state: u8) {
        match self.voxel_world_pos {
            None => {}
            Some(_) => {
                if let Some((last_chunk, last_voxel)) = self.last_position {
                    world.chunks[last_chunk].chunk.voxels[last_voxel] = self.last_state.unwrap();
                    world.voxels[last_chunk][last_voxel] = self.last_state.unwrap();
                }
                if world.voxels[self.chunk_index.unwrap()][self.voxel_index.unwrap()] == 0 {
                    return;
                }
                self.last_state =
                    Some(world.voxels[self.chunk_index.unwrap()][self.voxel_index.unwrap()]);
                world.voxels[self.chunk_index.unwrap()][self.voxel_index.unwrap()] = state;

                world.chunks[self.chunk_index.unwrap()].chunk.voxels[self.voxel_index.unwrap()] =
                    state;
                self.last_position = Some((self.chunk_index.unwrap(), self.voxel_index.unwrap()));
                self.last_world = Some(self.voxel_world_pos.unwrap());
            }
        }
    }

    pub fn select_voxel(&mut self, world: &mut World) -> usize {
        match self.voxel_world_pos {
            None => {return 0;}
            Some(_) => {
                if let Some((last_chunk, last_voxel)) = self.last_position {
                    world.chunks[last_chunk].chunk.voxels[last_voxel] = self.last_state.unwrap();
                    world.voxels[last_chunk][last_voxel] = self.last_state.unwrap();
                }
                if world.voxels[self.chunk_index.unwrap()][self.voxel_index.unwrap()] == 0 {
                    return 0;
                }
                self.last_state =
                    Some(world.voxels[self.chunk_index.unwrap()][self.voxel_index.unwrap()]);
                self.last_position = Some((self.chunk_index.unwrap(), self.voxel_index.unwrap()));
                self.last_world = Some(self.voxel_world_pos.unwrap());
                return 1;
            }
        }
    }

    pub fn add_voxel(&mut self, world: &mut World, state: u8) {
        use self::Direction::*;
        match self.voxel_world_pos {
            None => {}
            Some(_) => {
                let [dx, dy, dz] = match self.direction.unwrap() {
                    X(d) => [d, 0.0, 0.0],
                    Y(d) => [0.0, d, 0.0],
                    Z(d) => [0.0, 0.0, d],
                };

                let [mut x, mut y, mut z] = self.voxel_world_pos.unwrap();
                x = x + dx;
                y = y + dy;
                z = z + dz;
                if !(x < 0.0
                    || x >= WORLD_W as f32 * CHUNK_SIZE as f32
                    || y < 0.0
                    || y >= WORLD_H as f32 * CHUNK_SIZE as f32
                    || z < 0.0
                    || z >= WORLD_D as f32 * CHUNK_SIZE as f32)
                {
                    let (voxel_index, chunk_index) = get_voxel(x, y, z);
                    world.voxels[chunk_index][voxel_index] = state;

                    world.chunks[chunk_index].chunk.voxels[voxel_index] = state;
                }
            }
        }
    }
}

fn get_voxel(x: f32, y: f32, z: f32) -> (usize, usize) {
    let [cx, cy, cz] = [
        x as u32 / CHUNK_SIZE as u32,
        y as u32 / CHUNK_SIZE as u32,
        z as u32 / CHUNK_SIZE as u32,
    ];
    let [x, y, z] = [
        x - cx as f32 * CHUNK_SIZE as f32,
        y - cy as f32 * CHUNK_SIZE as f32,
        z - cz as f32 * CHUNK_SIZE as f32,
    ];
    let voxel_index =
        x as usize + CHUNK_SIZE as usize * z as usize + CHUNK_AREA as usize * y as usize;
    let chunk_index =
        cx as usize + WORLD_D as usize * cz as usize + WORLD_AREA as usize * cy as usize;
    (voxel_index, chunk_index)
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Z(f32),
    Y(f32),
    X(f32),
}
