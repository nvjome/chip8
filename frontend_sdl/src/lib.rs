use core::{CPU, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::error;
use sdl2;

const DISPLAY_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * DISPLAY_SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * DISPLAY_SCALE;

pub struct GameSDL {
    cpu: CPU,
    window: sdl2::video::Window,
}

pub fn init_frontend(rom_buffer: Vec<u8>) -> Result<GameSDL, Box<dyn error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()?;

    let mut game = GameSDL {
        cpu: CPU::new(),
        window: window
    };

    game.cpu.load_rom_from_buffer(&rom_buffer)?;
    
    Ok(game)
}

pub fn run_game(game: GameSDL) -> Result<(), Box<dyn error::Error>> {
    let mut canvas = game.window.into_canvas().present_vsync().build()?;
    canvas.clear();
    canvas.present();

    Ok(())
}