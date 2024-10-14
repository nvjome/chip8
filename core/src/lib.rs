use std::error;

mod fonts;

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
        Self {
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            index_register: 0,
            v_register: [0; NUM_REGISTERS],
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0,
            sound_timer: 0,
            display_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            key_states: [false; NUM_KEYS],
        }
    }

    pub fn load(&self) -> Result<(), Box<dyn error::Error>> {
        // Load ROM contents into RAM, starting at 0x200
        todo!()
    }

    pub fn cycle(&mut self) {
        let op_code = self.fetch();
        self.execute(op_code);
    }

    pub fn fetch(&mut self) -> u16 {
        // Program could panic here if program_counter is higher than ram.len()
        if ((self.program_counter + 1) as usize) >= RAM_SIZE {
            panic!("Program counter exceeds RAM size");
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
        
        opcode
    }

    fn execute(&mut self, op_code: u16) {
        todo!()
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn fetch_code() {
        let mut cpu = CPU::new();
        cpu.ram[0] = 0xDE;
        cpu.ram[1] = 0xAD;
        cpu.ram[2] = 0xBE;
        cpu.ram[3] = 0xEF;
        cpu.program_counter = 2;

        let code = cpu.fetch();
        assert_eq!(code, 0xBEEF);
    }

    #[test]
    #[should_panic(expected = "Program counter exceeds RAM size")]
    fn fetch_out_of_bounds() {
        let mut cpu = CPU::new();
        cpu.program_counter = RAM_SIZE as u16;

        let code = cpu.fetch();
    }
}
