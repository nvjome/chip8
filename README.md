# CHIP-8 intepreter

A CHIP-8 interpreter, written for the purpose of learning a little about Rust. 

This repo contains a Rust workspace, which separates the implementation of the backend (core) and frontend (frontend_sdl). A new frontend could be developed with a different graphics library, although I probably never will since SDL2 worked well for me here.

## Dependencies

This project requires SDL2 to build and run. Follow the instructions on the (rust-sdl2)[https://github.com/Rust-SDL2/rust-sdl2] package GitHub page.

## Quickstart

Clone repo, open the repo directory, then use `cargo run`:

`cargo run <path/to/rom> <cycles per frame> <total cycles>`

The first argument is the path to the ROM you want to load. The second is how many instructions to run per frame (10 is a good start). The third (optional) is how many CPU cycles to run before halting. This is useful for running test ROMs that may specify how many cycles to run to check specific instructions. Use 0 or omit to let the interpreter run normally.

### Note

The original CHIP-8 interpreter expected a 60Hz display refresh rate, so this interpreter expects the same to have accurate timers. If your monitor/display is higher than 60Hz things will run too fast. The "cycles per frame" argument is one way to control that, but the timers will still tick at the refresh rate. A more robust timer ticking mechanism is possible, perhaps by driving the timer tick from system time instead of frame rate, but not implemented here. For now, just run at 60Hz.