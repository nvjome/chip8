# CHIP-8 intepreter

A CHIP-8 interpreter, written for the purpose of learning a little about Rust. 

This repo contains a Rust workspace, which separates the implementation of the backend (core) and frontend (frontend_sdl). A new frontend could be developed with a different graphics library, although I probably never will since SDL2 worked well for me here.

Thi interpreter targets the "original" behavior of the CIP-8 interpreter for the COSMAC VIP. To that end, all functionality was developed against the test quite provided by [Tomendus](https://github.com/Timendus/chip8-test-suite), including the various quirks in instruction interpretations. There are many sources for the behavior of the instructions, and many have errors or inconsistencies, probably due to the variation in behavior seen on different systems the interpreter was ported to.

## Dependencies

This project requires SDL2 to build and run. Follow the instructions on the [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) GitHub page.

## Quickstart

Clone repo, open the repo directory, then use `cargo run`:

`cargo run <path/to/rom> <cycles per frame> <total cycles>`

The first argument is the path to the ROM you want to load. The second is how many instructions to run per frame (10 is a good start). The third (optional) is how many CPU cycles to run before halting. This is useful for running test ROMs that may specify how many cycles to run to check specific instructions. Use 0 or omit to let the interpreter run normally.

## A Note On Refresh Rate

The original CHIP-8 interpreter expected a 60Hz display refresh rate, so this interpreter expects the same to have accurate timers. If your monitor/display is higher than 60Hz things will run too fast. The "cycles per frame" argument is one way to control that, but the timers will still tick at the refresh rate. A more robust timer ticking mechanism is possible, perhaps by driving the timer tick from system time instead of frame rate, but not implemented here. For now, just run at 60Hz.
