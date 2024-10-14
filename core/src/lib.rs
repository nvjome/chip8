mod fonts;
mod core_error;

use std::error;
use std::fs::File;
use std::io::Read;

use crate::core_error::CoreError;
use crate::fonts::FONT_SET_1;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const START_ADDRESS: u16 = 0x200;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct CPU {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    index_register: u16,
    v_register: [u8; NUM_REGISTERS],
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    display_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    key_states: [bool; NUM_KEYS],
}

impl CPU {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            index_register: 0,
            v_register: [0; NUM_REGISTERS],
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0,
            sound_timer: 0,
            display_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            key_states: [false; NUM_KEYS],
        };

        new_cpu.ram[..FONT_SET_1.len()].copy_from_slice(&FONT_SET_1);
        new_cpu
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.index_register = 0;
        self.v_register = [0; NUM_REGISTERS];
        self.stack = Vec::with_capacity(STACK_SIZE);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.display_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.key_states = [false; NUM_KEYS];
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn error::Error>> {
        // Load ROM contents into RAM, starting at 0x200
        let mut rom_file = File::open(path)?;
        let mut rom_buffer = Vec::new();
        rom_file.read_to_end(&mut rom_buffer)?;

        self.load_rom_to_ram(&rom_buffer)?;

        Ok(())
    }

    fn load_rom_to_ram(&mut self, data: &[u8]) -> Result<(), CoreError> {
        // Check if ROM fits within available program space
        if data.len() > 0xA00 {
            return Err(CoreError::RomSizeError);
        }

        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);

        Ok(())
    }

    pub fn cycle(&mut self) -> Result<(), CoreError> {
        let op_code = self.fetch()?;
        self.execute(op_code);
        Ok(())
    }

    fn fetch(&mut self) -> Result<u16, CoreError> {
        // Program could panic here if program_counter is higher than ram.len()
        // Instead, return ProgramCounterError
        if ((self.program_counter + 1) as usize) >= RAM_SIZE {
            // panic!("Program counter exceeds RAM size");
             return Err(CoreError::ProgramCounterError { index: self.program_counter });
        }

        let upper_byte = self.ram[self.program_counter as usize];
        let lower_byte = self.ram[(self.program_counter + 1) as usize];

        // I tried to use ram.get and a slice for 2 bytes, but couldn't get it to work...
        /*
        let index = self.program_counter as usize;
        
        let bytes = match self.ram.get(index..index+1) {
            Some(b) => b,
            None => panic!("Program exceeds RAM size"),
        };

        let opcode = (bytes[0] as u16) << 8 | (bytes[1] as u16);
        */

        let opcode: u16 = (upper_byte as u16) << 8 | lower_byte as u16;
        self.program_counter += 2;
        
        Ok(opcode)
    }

    fn execute(&mut self, op_code: u16) {
        match op_code {
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_opcode() {
        let mut cpu = CPU::new();
        cpu.ram[0] = 0xDE;
        cpu.ram[1] = 0xAD;
        cpu.ram[2] = 0xBE;
        cpu.ram[3] = 0xEF;
        cpu.program_counter = 2;
        let code = cpu.fetch().unwrap();
        assert_eq!(code, 0xBEEF);
    }

    #[test]
    fn fetch_opcode_bounds() {
        let mut cpu = CPU::new();
        cpu.program_counter = RAM_SIZE as u16;
        assert!(cpu.fetch().is_err());
    }

    #[test]
    fn load_small_rom() {
        let mut cpu = CPU::new();
        let rom: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        let _ = cpu.load_rom_to_ram(&rom);
        let code = cpu.fetch().unwrap();
        assert_eq!(code, 0xDEAD);
    }

    #[test]
    fn load_large_rom() {
        let mut cpu = CPU::new();
        // Try to load a ROM as large as the RAM
        let rom: [u8; RAM_SIZE] = [0xA; RAM_SIZE];
        let load_result = cpu.load_rom_to_ram(&rom);
        print!("{:?}", load_result);
        assert!(load_result.is_err());
    }
}
