use core::CPU;
use std::{env, process};

fn main() {
    println!("Hello, world!");

    // Collect command line arguments
    let args: Vec<String> = env::args().collect();

    // Create CPU instance
    let mut cpu = CPU::new();

    // Create fontend instance and pass in PCU

    // Run frontend loop, exiting in case of errors
}
