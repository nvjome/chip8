use core::*;
use std::error;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window
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
    ticks_per_frame: u32,
    run_cycles: u32,
}

pub fn init_frontend(rom_buffer: Vec<u8>, ticks_per_frame: u32, run_cycles: u32) -> Result<GameSDL, Box<dyn error::Error>> {
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
        ticks_per_frame: ticks_per_frame,
        run_cycles: run_cycles,
    };

    game.cpu.load_rom_from_buffer(&rom_buffer)?;
    
    Ok(game)
}

pub fn run_game(game: &mut GameSDL) -> Result<(), Box<dyn error::Error>> {
    let mut event_pump = game.context.event_pump()?;
    let mut cycles = 0;

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(key), ..} => {
                    if let Some(k) = key_to_button(key) {
                        game.cpu.keypress(k, true)?;
                    }
                },
                Event::KeyUp{keycode: Some(key), ..} => {
                    if let Some(k) = key_to_button(key) {
                        game.cpu.keypress(k, false)?;
                    }
                }
                _ => (),
            }
        }

        if (cycles < game.run_cycles) || (game.run_cycles == 0) {
            for _ in 0..game.ticks_per_frame {
                game.cpu.cycle()?;
                // If run_cycles is non-zero, run only that many cycles
                if game.run_cycles > 0 {
                    cycles += 1;
                }
            }
        }

        game.cpu.tick_timers();
        draw_screen(&game.cpu, &mut game.canvas)?;
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

fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 =>    Some(0x1),
        Keycode::Num2 =>    Some(0x2),
        Keycode::Num3 =>    Some(0x3),
        Keycode::Num4 =>    Some(0xC),
        Keycode::Q =>       Some(0x4),
        Keycode::W =>       Some(0x5),
        Keycode::E =>       Some(0x6),
        Keycode::R =>       Some(0xD),
        Keycode::A =>       Some(0x7),
        Keycode::S =>       Some(0x8),
        Keycode::D =>       Some(0x9),
        Keycode::F =>       Some(0xE),
        Keycode::Z =>       Some(0xA),
        Keycode::X =>       Some(0x0),
        Keycode::C =>       Some(0xB),
        Keycode::V =>       Some(0xF),
        _ =>                None,
    }
}