mod fonts;
mod core_error;

use std::error;
use std::fs::File;
use std::io::Read;
use rand;

use crate::core_error::CoreError;
use crate::fonts::FONT_SET_1;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const SCREEN_BUFF_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

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
    display_buffer: [bool; SCREEN_BUFF_SIZE],
    display_update_flag: bool,
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
            display_buffer: [false; SCREEN_BUFF_SIZE],
            display_update_flag: false,
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
        self.display_buffer = [false; SCREEN_BUFF_SIZE];
        self.display_update_flag = false;
        self.key_states = [false; NUM_KEYS];

        self.ram[..FONT_SET_1.len()].copy_from_slice(&FONT_SET_1);
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

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Tone not implemented
            }

            self.sound_timer -= 1;
        }
    }

    pub fn cycle(&mut self) -> Result<(), CoreError> {
        let op_code = self.fetch()?;
        self.execute(op_code)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u16, CoreError> {
        // Program would panic if program_counter is higher than ram.len()
        // Instead, check the bounds and return ProgramCounterError in case of problem
        if ((self.program_counter + 1) as usize) >= RAM_SIZE {
             return Err(CoreError::ProgramCounterError { index: self.program_counter });
        }

        let upper_byte = self.ram[self.program_counter as usize];
        let lower_byte = self.ram[(self.program_counter + 1) as usize];

        let opcode: u16 = (upper_byte as u16) << 8 | lower_byte as u16;
        self.program_counter += 2;
        
        Ok(opcode)
    }

    fn execute(&mut self, op_code: u16) -> Result<(), CoreError> {
        let (nib4, nib3, nib2, nib1) = slice_u16(op_code);
        let  mut execute_result = Ok(());

        match (nib4, nib3, nib2, nib1) {
            (0, 0, 0, 0) => (), // NOP

            (0, 0, 0xE, 0) => { // Clear screen
                self.display_buffer = [false; SCREEN_BUFF_SIZE];
                self.display_update_flag = true;
            },

            (0, 0, 0xE, 0xE) => { // Return (exit subroutine)
                let stack_pop = self.stack.pop();
                match stack_pop {
                    Some(addr) => self.program_counter = addr,
                    None => execute_result = Err(CoreError::StackEmptyError),
                }
            },

            (0x1, _, _, _) => self.program_counter = op_code & 0x0FFF, // Jump

            (0x2, _, _, _) => { // Call subroutine
                self.stack.push(self.program_counter);
                self.program_counter = op_code & 0x0FFF;
            },

            (0x3, x, _, _) => { // Skip if vx == NN
                if self.v_register[x as usize] == (op_code & 0x00FF) as u8 {self.program_counter += 2};
            },

            (0x4, x, _, _) => { // Skip if vx != NN
                if self.v_register[x as usize] != (op_code & 0x00FF) as u8 {self.program_counter += 2};
            },

            (0x5, x, y, 0) => { // Skip if vx == vy
                if self.v_register[x as usize] == self.v_register[y as usize] {self.program_counter += 2};
            },

            (0x6, x, _, _) => self.v_register[x as usize] = (op_code & 0x00FF) as u8, // Store NN in vx

            (0x7, x, _, _) => self.v_register[x as usize] += (op_code & 0x00FF) as u8, // Add NN to vx

            (0x8, x, y, 0) => self.v_register[x as usize] = self.v_register[y as usize], // Store vy in vx

            (0x8, x, y, 0x1) => self.v_register[x as usize] = self.v_register[x as usize] | self.v_register[y as usize], // Store vx OR vy in vx

            (0x8, x, y, 0x2) => self.v_register[x as usize] = self.v_register[x as usize] & self.v_register[y as usize], // Store vx AND vy in vx

            (0x8, x, y, 0x3) => self.v_register[x as usize] = self.v_register[x as usize] ^ self.v_register[y as usize], // Store vx XOR vy in vx

            (0x8, x, y, 0x4) => { // Store vx + vy in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[x as usize].overflowing_add(self.v_register[y as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  0x1,
                    false => 0,
                };
            },

            (0x8, x, y, 0x5) => { // Store vx - vy in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[x as usize].overflowing_sub(self.v_register[y as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  0,
                    false => 0x1,
                };
            },

            (0x8, x, y, 0x6) => { // Set vf to LSB of vy, store vy >> 1 in vx
                self.v_register[0xf] = self.v_register[y as usize] & 0x01;
                self.v_register[x as usize] = self.v_register[y as usize] >> 1;
            },

            (0x8, x, y, 0x7) => { // Store vy - vx in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[y as usize].overflowing_sub(self.v_register[x as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  0,
                    false => 0x1,
                };
            },

            (0x8, x, y, 0xE) => { // Set vf to MSB of vy, store vy << 1 in vx
                self.v_register[0xf] = (self.v_register[y as usize] & 0x80) >> 7;
                self.v_register[x as usize] = self.v_register[y as usize] << 1;
            },

            (0x9, x, y, 0) => { // Skip if vx != vy
                if self.v_register[x as usize] != self.v_register[y as usize] {self.program_counter += 2};
            },

            (0xA, _, _, _) => self.index_register = op_code & 0x0FFF, // Set i to NNN

            (0xB, _, _, _) => self.index_register = (op_code & 0x0FFF) + self.v_register[0] as u16, // Set i to NNN + v0

            (0xC, x, _, _) => { // Set vx to random number 0-255, mask with NN
                let random_number = rand::random::<u8>();
                let mask = (op_code & 0x00FF) as u8;
                self.v_register[x as usize] = random_number & mask;
            },

            (0xE, x, 0x9, 0xE) => { // Skip if vx key is pressed
                let key = self.v_register[x as usize] as usize;
                if self.key_states[key] {
                    self.program_counter += 2;
                }
            },

            (0xE, x, 0xA, 0x1) => { // Skip if vx key is not pressed
                let key = self.v_register[x as usize] as usize;
                if self.key_states[key] == false {
                    self.program_counter += 2;
                }
            },

            (_, _, _, _) => execute_result = Err(CoreError::OpcodeError { opcode: (op_code) }),
        }

        return execute_result;
    }
}

// Slice u16 word into 4-bit nibbles, returned MSB first
fn slice_u16(word: u16) -> (u16, u16, u16, u16) {
    let n4 = (word & 0xF000) >> 12;
    let n3 = (word & 0x0F00) >> 8;
    let n2 = (word & 0x00F0) >> 4;
    let n1 = word & 0x000F;

    (n4, n3, n2, n1)
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
    #[test]
    fn slice_u16_test() {
        let word: u16 = 0xDEAD;
        let (n1, n2, n3, n4) = slice_u16(word);
        assert_eq!(n1, 0xD);
        assert_eq!(n2, 0xE);
        assert_eq!(n3, 0xA);
        assert_eq!(n4, 0xD);
    }

    #[test]
    fn op_00e0() {
        let mut cpu = CPU::new();
        cpu.display_buffer = [true; SCREEN_BUFF_SIZE];
        let _ = cpu.execute(0x00E0);
        assert_eq!(cpu.display_buffer[SCREEN_WIDTH], false);
        assert!(cpu.display_update_flag);
    }

    #[test]
    fn op_00ee() {
        let mut cpu = CPU::new();
        cpu.stack.push(0x0210);
        let _ = cpu.execute(0x00EE);
        assert_eq!(cpu.program_counter, 0x0210);
        assert!(cpu.stack.len() == 0);
    }
    
    #[test]
    fn op_1nnn() {
        let mut cpu = CPU::new();
        let _ = cpu.execute(0x1234);
        assert_eq!(cpu.program_counter, 0x0234);
    }

    #[test]
    fn op_2nnn() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0222;
        let _ = cpu.execute(0x2234);
        assert_eq!(cpu.program_counter, 0x0234);
        assert_eq!(cpu.stack[0], 0x0222);
    }

    #[test]
    fn op_3xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        let _ = cpu.execute(0x30AB);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_4xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        let _ = cpu.execute(0x40AC);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_5xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        cpu.v_register[1] = 0xAB; // vy
        let _ = cpu.execute(0x5010);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_6xnn() {
        let mut cpu = CPU::new();
        let _ = cpu.execute(0x65AB);
        assert_eq!(cpu.v_register[5], 0xAB);
    }

    #[test]
    fn op_7xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[5] = 0x04;
        let _ = cpu.execute(0x75AB);
        assert_eq!(cpu.v_register[5], 0xAF);
    }

    #[test]
    fn op_8xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        let _ = cpu.execute(0x8010);
        assert_eq!(cpu.v_register[0], cpu.v_register[1]);
    }

    #[test]
    fn op_8xy1() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x55; // vx
        cpu.v_register[1] = 0xAA; // vy
        let _ = cpu.execute(0x8011);
        assert_eq!(cpu.v_register[0], 0xFF);
    }

    #[test]
    fn op_8xy2() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x55; // vx
        cpu.v_register[1] = 0xAA; // vy
        let _ = cpu.execute(0x8012);
        assert_eq!(cpu.v_register[0], 0x00);
    }

    #[test]
    fn op_8xy3() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        let _ = cpu.execute(0x8013);
        assert_eq!(cpu.v_register[0], 0xF0);
    }

    #[test]
    fn op_8xy4() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        let _ = cpu.execute(0x8014);
        assert_eq!(cpu.v_register[0], 0x0E);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_8xy5() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        let _ = cpu.execute(0x8015);
        assert_eq!(cpu.v_register[0], 0xB0);
        assert_eq!(cpu.v_register[0xf], 0x00);
    }

    #[test]
    fn op_8xy6() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        let _ = cpu.execute(0x8016);
        assert_eq!(cpu.v_register[0], 0x55);
        assert_eq!(cpu.v_register[1], 0xAB);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_8xy7() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAF; // vx
        cpu.v_register[1] = 0x5F; // vy
        let _ = cpu.execute(0x8017);
        assert_eq!(cpu.v_register[0], 0xB0);
        assert_eq!(cpu.v_register[0xf], 0x00);
    }

    #[test]
    fn op_8xye() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        let _ = cpu.execute(0x801E);
        assert_eq!(cpu.v_register[0], 0x56);
        assert_eq!(cpu.v_register[1], 0xAB);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_9xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        cpu.v_register[1] = 0xAC; // vy
        let _ = cpu.execute(0x9010);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_annn() {
        let mut cpu = CPU::new();
        let _ = cpu.execute(0xA321);
        assert_eq!(cpu.index_register, 0x0321);
    }

    #[test]
    fn op_bnnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x0010;
        let _ = cpu.execute(0xB321);
        assert_eq!(cpu.index_register, 0x0331);
    }

    #[test]
    fn op_cxnn() {
        let mut cpu = CPU::new();
        let _ = cpu.execute(0xcAFF);
        println!("{}", cpu.v_register[0xA]);
    }

    #[test]
    fn op_ex9e() {
        let mut cpu = CPU::new();
        cpu.key_states[0x7] = true; // Key 7 pressed
        cpu.v_register[0] = 0x7; // Key 7 in v0
        cpu.v_register[1] = 0x4; // Key 4 in v1
        let _ = cpu.execute(0xE09E);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
        let _ = cpu.execute(0xE19E);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_exa1() {
        let mut cpu = CPU::new();
        cpu.key_states[0x7] = true; // Key 7 pressed
        cpu.v_register[0] = 0x7; // Key 7 in v0
        cpu.v_register[1] = 0x4; // Key 4 in v1
        let _ = cpu.execute(0xE0A1);
        assert_eq!(cpu.program_counter, START_ADDRESS);
        let _ = cpu.execute(0xE1A1);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn timer_test() {
        let mut cpu = CPU::new();
        cpu.delay_timer = 5;
        cpu.sound_timer = 4;
        for _i in 0..4 {
            cpu.tick_timers();
        }
        assert_eq!(cpu.delay_timer, 1);
        assert_eq!(cpu.sound_timer, 0);
        cpu.tick_timers();
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);
    }
}
