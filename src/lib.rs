mod utils;
mod world;

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Receiver};
//use utils::model;
use utils::state::State;
use utils::{input::InputHandler, voxel_handler::VoxelHandler};
use world::chunk::CHUNK_SIZE;
use world::{WORLD_AREA, WORLD_D, WORLD_H, WORLD_W};

use std::time::{Duration, Instant};
use tokio::time::sleep;

use winit::{event_loop::EventLoop, keyboard::KeyCode, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

struct VoxelGame {
    world: Option<world::World>,
    mobs: Vec<utils::model::Model>,
    input_handler: InputHandler,
    voxel_handler: VoxelHandler,
    num_block: u8,
}

impl<'a> VoxelGame {
    fn new() -> Self {
        let mobs = vec![];
        let input_handler = InputHandler::new();
        Self {
            world: None,
            mobs,
            input_handler,
            voxel_handler: VoxelHandler::new(),
            num_block: 1,
        }
    }

    async fn run(
        mut game: VoxelGame,
        state: Arc<Mutex<State<'a>>>,
        mut input_rx: Receiver<Box<InputHandler>>,
    ) {
        let frame_duration = Duration::from_secs_f32(1.0 / FPS as f32);
        let mut last_frame_time = Instant::now();

        loop {
            let now = Instant::now();
            let delta_time = now.duration_since(last_frame_time);

            while let Ok(input) = input_rx.try_recv() {
                game.input_handler = *input;
            }
            if game.input_handler.close{
                break;
            }
            if delta_time >= frame_duration {
                //println!("{}", delta_time.as_millis());
                if let Ok(mut state) = state.lock() {
                    VoxelGame::render(&mut game, &mut state);
                    VoxelGame::update(&mut game, &mut state);
                    drop(state);
                }

                last_frame_time = now;
            }
            let elapsed = last_frame_time.elapsed();
            if elapsed < frame_duration {
                sleep(frame_duration - elapsed).await;
            }
        }
    }

    fn update(game: &mut VoxelGame, state: &mut State<'a>) {
        let mut camera = state.camera;
        let relation = state.camera_uniform.relation;
        let mouse_pos = game.input_handler.mouse_pos();
        let size = state.size;
        VoxelGame::input(game);
        state.input(&game.input_handler);
        let mut voxel_handler = game.voxel_handler.clone();
        if let Some(world) = &mut game.world {
            voxel_handler.update(&mut camera, mouse_pos, size, relation, world);

            if let Some(index) = voxel_handler.chunk_index {
                let select = voxel_handler.select_voxel(world);
                world.chunks[index].reflesh(&state.queue, &world.voxels, Some(select));

                if let Some(_) = voxel_handler.last_world {
                    let [x, y, z] = voxel_handler.last_world.unwrap();
                    let c1 = [
                        x as u32 / CHUNK_SIZE as u32,
                        y as u32 / CHUNK_SIZE as u32,
                        z as u32 / CHUNK_SIZE as u32,
                    ];

                    let [x, y, z] = voxel_handler.voxel_world_pos.unwrap();
                    let c2 = [
                        x as u32 / CHUNK_SIZE as u32,
                        y as u32 / CHUNK_SIZE as u32,
                        z as u32 / CHUNK_SIZE as u32,
                    ];

                    if c1 != c2 {
                        let [cx, cy, cz] = c1;
                        let chunk_index = cx as usize
                            + WORLD_D as usize * cz as usize
                            + WORLD_AREA as usize * cy as usize;
                        if let Some((last_chunk, last_voxel)) = voxel_handler.last_position {
                            world.chunks[last_chunk].chunk.voxels[last_voxel] =
                                voxel_handler.last_state.unwrap();
                            world.voxels[last_chunk][last_voxel] =
                                voxel_handler.last_state.unwrap();
                        }

                        world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                    }
                }

                if game.input_handler.check_mouse(
                    winit::event::MouseButton::Left,
                    utils::input::InputType::Pressed,
                ) {
                    voxel_handler.change_voxel(world, 0);
                    voxel_handler.last_state = Some(0);
                } else if game.input_handler.check_mouse(
                    winit::event::MouseButton::Right,
                    utils::input::InputType::Pressed,
                ) {
                    voxel_handler.add_voxel(world, game.num_block);
                }
                if voxel_handler.last_state == Some(0) {
                    let w_pos = voxel_handler.voxel_world_pos.unwrap();
                    let [x, y, z] = w_pos;
                    let [mut cx, mut cy, mut cz] = [
                        x as u32 / CHUNK_SIZE as u32,
                        y as u32 / CHUNK_SIZE as u32,
                        z as u32 / CHUNK_SIZE as u32,
                    ];
                    let mut chunk_index: usize;

                    let [x, y, z] = [x.floor(), y.floor(), z.floor()];

                    if x % CHUNK_SIZE as f32 == 0.0 {
                        if cx > 0 {
                            cx -= 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    } else if (x + 1.0) % CHUNK_SIZE as f32 - 1.0 == 0.0 {
                        if cx < WORLD_W {
                            cx += 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    }

                    if y % CHUNK_SIZE as f32 == 0.0 {
                        if cy > 0 {
                            cy -= 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    } else if (y + 1.0) % CHUNK_SIZE as f32 == 0.0 {
                        if cy < WORLD_H {
                            cy += 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    }

                    if z % CHUNK_SIZE as f32 == 0.0 {
                        if cz > 0 {
                            cz -= 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    } else if (z + 1.0) % CHUNK_SIZE as f32 == 0.0 {
                        if cz < WORLD_W {
                            cz += 1;
                            chunk_index = cx as usize
                                + WORLD_D as usize * cz as usize
                                + WORLD_AREA as usize * cy as usize;
                            world.chunks[chunk_index].reflesh(&state.queue, &world.voxels, None);
                        }
                    }
                }
            }
            world.update();
        }
        game.voxel_handler = voxel_handler;
    }

    fn input(game: &mut VoxelGame){
        if game.input_handler.check_key(KeyCode::Digit0, utils::input::InputType::Pressed){
            game.num_block = 1;
        }
        if game.input_handler.check_key(KeyCode::Digit1, utils::input::InputType::Pressed){
            game.num_block = 2;
        }
        if game.input_handler.check_key(KeyCode::Digit2, utils::input::InputType::Pressed){
            game.num_block = 3;
        }
        if game.input_handler.check_key(KeyCode::Digit3, utils::input::InputType::Pressed){
            game.num_block = 4;
        }
        if game.input_handler.check_key(KeyCode::Digit4, utils::input::InputType::Pressed){
            game.num_block = 5;
        }
        if game.input_handler.check_key(KeyCode::Digit5, utils::input::InputType::Pressed){
            game.num_block = 6;
        }
        if game.input_handler.check_key(KeyCode::Digit6, utils::input::InputType::Pressed){
            game.num_block = 7;
        }
        
    }

    fn render(game: &mut VoxelGame, state: &mut State<'a>) {
        state.window().request_redraw();
        let mobs = &game.mobs;
        let size = state.size;
        let camera_m = state.camera.build_view_projection_matrix();
        match &game.world {
            None => {}
            Some(world) => {
                match state.render(mobs, world, camera_m, state.camera_uniform.relation) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        panic!("Se te acabo la memoria mi rey");
                    }

                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        panic!("Eres muy lento mi rey")
                    }
                }
            }
        }
    }

    fn start_world(&mut self, state: &State<'a>) {
        let mut world = world::World::new();
        let bytes = include_bytes!("assets/tex_array_0.png");
        world.build_chunk(
            &state.device,
            bytes,
            &state.texture_bind_group_layout,
            &state.queue,
        );
        self.world = Some(world);
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

//Game
const FPS: u64 = 120;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Isocraft");
    let state = State::new(Arc::new(window)).await;
    let mut game = VoxelGame::new();
    game.start_world(&state);
    let (input_tx, input_rx) = mpsc::channel::<Box<InputHandler>>(32);
    let state = Arc::new(Mutex::new(state));
    let state_copy = Arc::clone(&state);
    let render_thread =
        tokio::spawn(async move { VoxelGame::run(game, state_copy, input_rx).await });

    event_loop
        .run(|event, control_flow| {
            if input.update(&event) {
                let mut input_handler = InputHandler::new();
                input_handler.update(&input);
                if let Err(e) = input_tx.try_send(Box::new(input_handler)) {
                    eprintln!("{}", e);
                }
            }
            if input.close_requested() || input.key_pressed(KeyCode::Escape) || input.destroyed() {
                let mut input_handler = InputHandler::new();
                input_handler.close = true;
                if let Err(e) = input_tx.try_send(Box::new(input_handler)) {
                    eprintln!("{}", e);
                }
                render_thread.abort();
                control_flow.exit();
            }

            if let Some(physical_size) = input.window_resized() {
                if let Ok(mut state) = state.lock() {
                    state.resize(physical_size);
                }
            };
        })
        .unwrap();
}
