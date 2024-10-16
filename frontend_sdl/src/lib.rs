use core::*;
use std::error;
use sdl2::{
    event::Event,
    pixels::Color,
    rect::{self, Rect},
    render::Canvas,
    video::Window
};

const DISPLAY_SCALE: u32 = 10;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * DISPLAY_SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * DISPLAY_SCALE;

pub struct GameSDL {
    cpu: CPU,
    context: sdl2::Sdl,
    subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    canvas: sdl2::render::Canvas<Window>,
}

pub fn init_frontend(rom_buffer: Vec<u8>) -> Result<GameSDL, Box<dyn error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Create window
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()?;

    // Create canvas in window
    let canvas = window.clone().into_canvas().present_vsync().build()?;

    let mut game = GameSDL {
        cpu: CPU::new(),
        context: sdl_context,
        subsystem: video_subsystem,
        window: window,
        canvas: canvas,
    };

    game.cpu.load_rom_from_buffer(&rom_buffer)?;
    
    Ok(game)
}

pub fn run_game(game: &mut GameSDL) -> Result<(), Box<dyn error::Error>> {
    /*
    game.canvas.clear();
    game.canvas.present();

    game.canvas.set_draw_color(Color::RGB(255, 255, 255));

    
    let mut r = Rect::new(0, 0, DISPLAY_SCALE, DISPLAY_SCALE);
    game.canvas.fill_rect(r)?;
    r = Rect::new(DISPLAY_SCALE as i32, DISPLAY_SCALE as i32, DISPLAY_SCALE, DISPLAY_SCALE);
    game.canvas.fill_rect(r)?;

    r = Rect::new((SCREEN_WIDTH as i32 - 1) * DISPLAY_SCALE as i32, (SCREEN_HEIGHT as i32 - 1) * DISPLAY_SCALE as i32, DISPLAY_SCALE, DISPLAY_SCALE);
    game.canvas.fill_rect(r)?;
    r = Rect::new((SCREEN_WIDTH as i32 - 2) * DISPLAY_SCALE as i32, (SCREEN_HEIGHT as i32 - 2) * DISPLAY_SCALE as i32, DISPLAY_SCALE, DISPLAY_SCALE);
    game.canvas.fill_rect(r)?;
  
    for i in 0..(SCREEN_WIDTH * 2) {
        let x = (i % SCREEN_WIDTH) as u32;
        let y = (i / SCREEN_WIDTH) as u32;

        if true {
            let r = Rect::new((x * DISPLAY_SCALE) as i32, (y * DISPLAY_SCALE) as i32, DISPLAY_SCALE, DISPLAY_SCALE);
            game.canvas.fill_rect(r)?;
        }
    }
    game.canvas.present();
    */

    let mut event_pump = game.context.event_pump()?;

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                },
                _ => (),
            }
        }

        game.cpu.cycle()?;
        
        if game.cpu.display_update_flag {
            draw_screen(&game.cpu, &mut game.canvas)?;
            game.cpu.display_update_flag = false;
        }
    }

    Ok(())
}

fn draw_screen(cpu: &CPU, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn std::error::Error>> {
    // Clear canvas with black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buffer = cpu.get_display();
    
    // Draw color white
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    // Draw rectangles as pixels
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            // Get (x,y) from i and screen dimensions
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // Draw rectangle
            canvas.fill_rect(Rect::new((x * DISPLAY_SCALE) as i32, (y * DISPLAY_SCALE) as i32, DISPLAY_SCALE, DISPLAY_SCALE))?;
        }
    }

    canvas.present();
    Ok(())
}