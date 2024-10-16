use std::{env, process};
use frontend_sdl;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Hello, world!");

    // Collect command line arguments
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run /path/to/rom");
        process::exit(1);
    }

    // Open ROM file
    let mut rom_file = match File::open(args[2].clone()) {
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
    let mut game_sdl = match frontend_sdl::init_frontend(rom_buffer) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Error starting frontend: {}", e);
            process::exit(1);
        },
    };

    // Run frontend loop, exiting in case of errors
    if let Err(e) = frontend_sdl::run_game(game_sdl) {
        eprintln!("Error running game: {}", e);
        process::exit(1);
    }
}
