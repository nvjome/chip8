use std::{env, process};
use frontend_sdl;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Hello, world!");

    // Collect command line arguments
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run </path/to/rom> <cycles per frame> <cycles to run> ");
        process::exit(1);
    }

    let cycles_per_frame = match args[2].clone().parse::<u32>() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("Failed to parse arguments: {}", err);
            process::exit(1);
        }
    };

    let cycles = match args[3].clone().parse::<u32>() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("Failed to parse arguments: {}", err);
            process::exit(1);
        }
    };

    println!("Loading file: {}", args[1].clone());

    // Open ROM file
    let mut rom_file = match File::open(args[1].clone()) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening file: {}", err);
            process::exit(1);
        },
    };

    let mut rom_buffer = Vec::new();
    if let Err(e) = rom_file.read_to_end(&mut rom_buffer) {
        eprintln!("Error reading ROM: {}", e);
        process::exit(1);
    }

    // Create fontend instance
    let mut game_sdl = match frontend_sdl::init_frontend(rom_buffer, cycles_per_frame, cycles) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Error starting frontend: {}", e);
            process::exit(1);
        },
    };

    // Run frontend loop, exiting in case of errors
    if let Err(e) = frontend_sdl::run_game(&mut game_sdl) {
        eprintln!("Error running game: {}", e);
        process::exit(1);
    }
}
