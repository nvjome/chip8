mod fonts;
mod core_error;

use std::error;

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
const FONT_ADDRESS_OFFSET: u16 = 0;
const SPRITE_WIDTH: usize = 8;
const SPRITE_BYTES_MAX: usize = 15;

pub struct CPU {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    index_register: u16,
    v_register: [u8; NUM_REGISTERS],
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    display_buffer: [bool; SCREEN_BUFF_SIZE],
    pub display_update_flag: bool,
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

        new_cpu.load_font(&FONT_ADDRESS_OFFSET, &FONT_SET_1);
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

        // self.ram[(FONT_ADDRESS_OFFSET as usize)..(FONT_ADDRESS_OFFSET as usize) + FONT_SET_1.len()].copy_from_slice(&FONT_SET_1);
        self.load_font(&FONT_ADDRESS_OFFSET, &FONT_SET_1);
    }

    fn load_font(&mut self, offset: &u16, font: &[u8; 80]) {
        self.ram[(*offset as usize)..(*offset as usize + font.len())].copy_from_slice(font);
    }

    pub fn load_rom_from_buffer(&mut self, rom_buffer: &Vec<u8>) -> Result<(), Box<dyn error::Error>> {
        // Load ROM contents into RAM, starting at 0x200
        // let mut rom_file = File::open(path)?;
        // let mut rom_buffer = Vec::new();
        // rom_file.read_to_end(&mut rom_buffer)?;

        if rom_buffer.len() > 0xA00 {
            return Err(CoreError::RomSizeError.into());
        }

        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + rom_buffer.len();
        self.ram[start..end].copy_from_slice(&rom_buffer);

        Ok(())
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display_buffer
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) -> Result<(), CoreError> {
        if index > NUM_KEYS {
            return Err(CoreError::KeyIndexError { key: index });
        }
        self.key_states[index] = pressed;
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

            (1, _, _, _) => self.program_counter = op_code & 0x0FFF, // Jump

            (2, _, _, _) => { // Call subroutine
                self.stack.push(self.program_counter);
                self.program_counter = op_code & 0x0FFF;
            },

            (3, x, _, _) => { // Skip if vx == NN
                if self.v_register[x as usize] == (op_code & 0x00FF) as u8 {self.program_counter += 2};
            },

            (4, x, _, _) => { // Skip if vx != NN
                if self.v_register[x as usize] != (op_code & 0x00FF) as u8 {self.program_counter += 2};
            },

            (5, x, y, 0) => { // Skip if vx == vy
                if self.v_register[x as usize] == self.v_register[y as usize] {self.program_counter += 2};
            },

            (6, x, _, _) => self.v_register[x as usize] = (op_code & 0x00FF) as u8, // Store NN in vx

            (7, x, _, _) => self.v_register[x as usize] = self.v_register[x as usize].wrapping_add((op_code & 0x00FF) as u8), // Add NN to vx

            (8, x, y, 0) => self.v_register[x as usize] = self.v_register[y as usize], // Store vy in vx

            (8, x, y, 1) => self.v_register[x as usize] = self.v_register[x as usize] | self.v_register[y as usize], // Store vx OR vy in vx

            (8, x, y, 2) => self.v_register[x as usize] = self.v_register[x as usize] & self.v_register[y as usize], // Store vx AND vy in vx

            (8, x, y, 3) => self.v_register[x as usize] = self.v_register[x as usize] ^ self.v_register[y as usize], // Store vx XOR vy in vx

            (8, x, y, 4) => { // Store vx + vy in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[x as usize].overflowing_add(self.v_register[y as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  1,
                    false => 0,
                };
            },

            (8, x, y, 5) => { // Store vx - vy in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[x as usize].overflowing_sub(self.v_register[y as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  0,
                    false => 1,
                };
            },

            (8, x, y, 6) => { // Set vf to LSB of vy, store vy >> 1 in vx
                self.v_register[0xf] = self.v_register[y as usize] & 0x01;
                self.v_register[x as usize] = self.v_register[y as usize] >> 1;
            },

            (8, x, y, 0x7) => { // Store vy - vx in vx, set/unset carry flag vf
                let (sum, carry) = self.v_register[y as usize].overflowing_sub(self.v_register[x as usize]);
                self.v_register[x as usize] = sum;
                self.v_register[0xf] = match carry {
                    true =>  0,
                    false => 1,
                };
            },

            (8, x, y, 0xE) => { // Set vf to MSB of vy, store vy << 1 in vx
                self.v_register[0xf] = (self.v_register[y as usize] & 0x80) >> 7;
                self.v_register[x as usize] = self.v_register[y as usize] << 1;
            },

            (9, x, y, 0) => { // Skip if vx != vy
                if self.v_register[x as usize] != self.v_register[y as usize] {self.program_counter += 2};
            },

            (0xA, _, _, _) => self.index_register = op_code & 0x0FFF, // Set i to NNN

            (0xB, _, _, _) => self.index_register = (op_code & 0x0FFF).wrapping_add(self.v_register[0] as u16), // Set i to NNN + v0

            (0xC, x, _, _) => { // Set vx to random number 0-255, mask with NN
                let random_number = rand::random::<u8>();
                let mask = (op_code & 0x00FF) as u8;
                self.v_register[x as usize] = random_number & mask;
            },

            (0xD, x, y, n) => { // Draw n-byte sprite on screen at (vx,vy) starting at i, set vf if a pixel is erased
                // Get sprite coordinates
                let sprite_x = self.v_register[x as usize];
                let sprite_y = self.v_register[y as usize];
                let mut collide = false;

                // Copy sprite from RAM. Uses more memory than just reading from RAM, but should make code cleaner
                let mut sprite: Vec<u8> = Vec::with_capacity(SPRITE_BYTES_MAX);
                for i in 0..(n as usize) {
                    sprite.push(self.ram[self.index_register as usize + i]);
                }

                for byte_row in 0..(sprite.len() as u8) {
                    for bit_col in 0..(SPRITE_WIDTH as u8) {
                        if ((0b10000000 >> bit_col) & sprite[byte_row as usize]) != 0 { // Sprite pixel is 1
                            let pixel_x = (sprite_x + bit_col) as usize % SCREEN_WIDTH; // Wrap around screen
                            let pixel_y = (sprite_y + byte_row) as usize % SCREEN_HEIGHT;

                            let display_buffer_index = (pixel_y * SCREEN_WIDTH) + pixel_x;

                            collide |= self.display_buffer[display_buffer_index]; // If display pixel is already 1, then there is a collision
                            self.display_buffer[display_buffer_index] ^= true; // XOR sprite pixel and display pixel
                        }
                    }
                }

                match collide {
                    true => self.v_register[0xF] = 1,
                    false => self.v_register[0xF] = 0,
                }

                self.display_update_flag = true;
            },

            (0xE, x, 0x9, 0xE) => { // Skip if vx key is pressed
                let key = self.v_register[x as usize] as usize;
                if self.key_states[key] {
                    self.program_counter += 2;
                }
            },

            (0xE, x, 0xA, 1) => { // Skip if vx key is not pressed
                let key = self.v_register[x as usize] as usize;
                if self.key_states[key] == false {
                    self.program_counter += 2;
                }
            },

            (0xF, x, 0, 7) => self.v_register[x as usize] = self.delay_timer, // Set vx to value of delay timer

            (0xF, x, 0, 0xA) => { // Wait for a key press, store key in vx
                let mut wait = true;
                for i in 0..self.key_states.len() {
                    if self.key_states[i] {
                        wait = false;
                        self.v_register[x as usize] = i as u8;
                        break; // If multiple keys are pressed, this stops on the lowest indexed one
                    }
                }
                if wait {
                    self.program_counter -= 2;
                }
            },

            (0xF, x, 1, 5) => self.delay_timer = self.v_register[x as usize], // Set delay timer to value in vx

            (0xF, x, 1, 8) => self.sound_timer = self.v_register[x as usize], // Set sound timer to value in vx

            (0xF, x, 1, 0xE) => self.index_register = self.index_register.wrapping_add(self.v_register[x as usize] as u16), // Set i to i + vx

            (0xF, x, 2, 9) => self.index_register = (FONT_ADDRESS_OFFSET as u16) + (self.v_register[x as usize] as u16) * 5, // Set i to address of sprite for digit vx

            (0xF, x, 3, 3) => { // Decode BCD digits of vx and save to addresses i, i+1, and i+2
                let one = self.v_register[x as usize] % 10;
                let ten = self.v_register[x as usize] % 100 - one;
                let hundred = self.v_register[x as usize] - ten - one;
                self.ram[self.index_register as usize] = hundred / 100;
                self.ram[self.index_register as usize + 1] = ten / 10;
                self.ram[self.index_register as usize + 2] = one;
            },

            (0xF, x, 5, 5) => { // Copy v[0..=x] to ram[i..=i+x], set i to i+x+1
                if self.index_register + x >= RAM_SIZE as u16 {
                    return Err(CoreError::IndexRegisterError { index: (self.index_register + x) });
                }
                for i in 0..=(x as usize) {
                    self.ram[self.index_register as usize + i] = self.v_register[i];
                }
                self.index_register += x + 1;
            },

            (0xF, x, 6, 5) => { // Copy ram[i..=i+x] to v[0..=x], set i to i+x+1
                if self.index_register + x >= RAM_SIZE as u16 {
                    return Err(CoreError::IndexRegisterError { index: (self.index_register + x) });
                }
                for i in 0..=(x as usize) {
                    self.v_register[i] = self.ram[self.index_register as usize + i];
                }
                self.index_register += x + 1;
            },

            (_, _, _, _) => return Err(CoreError::OpcodeError { opcode: (op_code) }),
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

// It's [tests] all the way down!

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
        let rom = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(cpu.load_rom_from_buffer(&rom).is_ok());
        let code = cpu.fetch().unwrap();
        assert_eq!(code, 0xDEAD);
    }

    #[test]
    fn load_large_rom() {
        let mut cpu = CPU::new();
        // Try to load a ROM as large as the RAM
        let rom = vec![0xA; RAM_SIZE];
        let load_result = cpu.load_rom_from_buffer(&rom);
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

    #[test]
    fn op_00e0() {
        let mut cpu = CPU::new();
        cpu.display_buffer = [true; SCREEN_BUFF_SIZE];
        assert!(cpu.execute(0x00E0).is_ok());
        assert_eq!(cpu.display_buffer[SCREEN_WIDTH], false);
        assert!(cpu.display_update_flag);
    }

    #[test]
    fn op_00ee() {
        let mut cpu = CPU::new();
        cpu.stack.push(0x0210);
        assert!(cpu.execute(0x00EE).is_ok());
        assert_eq!(cpu.program_counter, 0x0210);
        assert!(cpu.stack.len() == 0);
    }
    
    #[test]
    fn op_1nnn() {
        let mut cpu = CPU::new();
        assert!(cpu.execute(0x1234).is_ok());
        assert_eq!(cpu.program_counter, 0x0234);
    }

    #[test]
    fn op_2nnn() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0222;
        assert!(cpu.execute(0x2234).is_ok());
        assert_eq!(cpu.program_counter, 0x0234);
        assert_eq!(cpu.stack[0], 0x0222);
    }

    #[test]
    fn op_3xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        assert!(cpu.execute(0x30AB).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_4xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        assert!(cpu.execute(0x40AC).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_5xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        cpu.v_register[1] = 0xAB; // vy
        assert!(cpu.execute(0x5010).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_6xnn() {
        let mut cpu = CPU::new();
        assert!(cpu.execute(0x65AB).is_ok());
        assert_eq!(cpu.v_register[5], 0xAB);
    }

    #[test]
    fn op_7xnn() {
        let mut cpu = CPU::new();
        cpu.v_register[5] = 0x04;
        assert!(cpu.execute(0x75AB).is_ok());
        assert_eq!(cpu.v_register[5], 0xAF);
    }

    #[test]
    fn op_8xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        assert!(cpu.execute(0x8010).is_ok());
        assert_eq!(cpu.v_register[0], cpu.v_register[1]);
    }

    #[test]
    fn op_8xy1() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x55; // vx
        cpu.v_register[1] = 0xAA; // vy
        assert!(cpu.execute(0x8011).is_ok());
        assert_eq!(cpu.v_register[0], 0xFF);
    }

    #[test]
    fn op_8xy2() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x55; // vx
        cpu.v_register[1] = 0xAA; // vy
        assert!(cpu.execute(0x8012).is_ok());
        assert_eq!(cpu.v_register[0], 0x00);
    }

    #[test]
    fn op_8xy3() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        assert!(cpu.execute(0x8013).is_ok());
        assert_eq!(cpu.v_register[0], 0xF0);
    }

    #[test]
    fn op_8xy4() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        assert!(cpu.execute(0x8014).is_ok());
        assert_eq!(cpu.v_register[0], 0x0E);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_8xy5() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x5F; // vx
        cpu.v_register[1] = 0xAF; // vy
        assert!(cpu.execute(0x8015).is_ok());
        assert_eq!(cpu.v_register[0], 0xB0);
        assert_eq!(cpu.v_register[0xf], 0x00);
    }

    #[test]
    fn op_8xy6() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        assert!(cpu.execute(0x8016).is_ok());
        assert_eq!(cpu.v_register[0], 0x55);
        assert_eq!(cpu.v_register[1], 0xAB);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_8xy7() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAF; // vx
        cpu.v_register[1] = 0x5F; // vy
        assert!(cpu.execute(0x8017).is_ok());
        assert_eq!(cpu.v_register[0], 0xB0);
        assert_eq!(cpu.v_register[0xf], 0x00);
    }

    #[test]
    fn op_8xye() {
        let mut cpu = CPU::new();
        cpu.v_register[1] = 0xAB; // vy
        assert!(cpu.execute(0x801E).is_ok());
        assert_eq!(cpu.v_register[0], 0x56);
        assert_eq!(cpu.v_register[1], 0xAB);
        assert_eq!(cpu.v_register[0xf], 0x01);
    }

    #[test]
    fn op_9xy0() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0xAB; // vx
        cpu.v_register[1] = 0xAC; // vy
        assert!(cpu.execute(0x9010).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_annn() {
        let mut cpu = CPU::new();
        assert!(cpu.execute(0xA321).is_ok());
        assert_eq!(cpu.index_register, 0x0321);
    }

    #[test]
    fn op_bnnn() {
        let mut cpu = CPU::new();
        cpu.v_register[0] = 0x0010;
        assert!(cpu.execute(0xB321).is_ok());
        assert_eq!(cpu.index_register, 0x0331);
    }

    #[test]
    fn op_cxnn() {
        let mut cpu = CPU::new();
        assert!(cpu.execute(0xcAFF).is_ok());
        println!("{}", cpu.v_register[0xA]);
    }

    #[test]
    fn op_dxyn() {
        let mut cpu = CPU::new();
        cpu.index_register = FONT_ADDRESS_OFFSET;
        assert!(cpu.execute(0xD015).is_ok());
        assert_eq!(cpu.display_buffer[0], true);
        assert_eq!(cpu.display_buffer[SCREEN_WIDTH + 1], false);
        assert_eq!(cpu.display_buffer[(SCREEN_WIDTH * 4) + 3], true);
        assert!(cpu.display_update_flag);
    }

    #[test]
    fn op_ex9e() {
        let mut cpu = CPU::new();
        cpu.key_states[0x7] = true; // Key 7 pressed
        cpu.v_register[0] = 0x7; // Key 7 in v0
        cpu.v_register[1] = 0x4; // Key 4 in v1
        assert!(cpu.execute(0xE09E).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
        assert!(cpu.execute(0xE19E).is_ok());
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
        assert!(cpu.execute(0xE1A1).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_fx07() {
        let mut cpu = CPU::new();
        cpu.delay_timer = 0xB2;
        assert!(cpu.execute(0xF707).is_ok());
        assert_eq!(cpu.v_register[7], 0xB2);
    }

    #[test]
    fn op_fx0a() {
        let mut cpu = CPU::new();
        assert!(cpu.fetch().is_ok()); // Need a fetch to increment program counter
        assert!(cpu.execute(0xF60A).is_ok());
        assert_eq!(cpu.program_counter, START_ADDRESS);
        cpu.key_states[9] = true;
        assert!(cpu.fetch().is_ok());  // Need a fetch to increment program counter
        assert!(cpu.execute(0xF60A).is_ok());
        assert_eq!(cpu.v_register[6], 9);
        assert_eq!(cpu.program_counter, START_ADDRESS + 2);
    }

    #[test]
    fn op_fx15() {
        let mut cpu = CPU::new();
        cpu.v_register[0xC] = 0xB2;
        assert!(cpu.execute(0xFC15).is_ok());
        assert_eq!(cpu.delay_timer, 0xB2);
    }

    #[test]
    fn op_fx18() {
        let mut cpu = CPU::new();
        cpu.v_register[0xC] = 0xB2;
        assert!(cpu.execute(0xFC18).is_ok());
        assert_eq!(cpu.sound_timer, 0xB2);
    }

    #[test]
    fn op_fx1e() {
        let mut cpu = CPU::new();
        cpu.v_register[0xC] = 0xB2;
        cpu.index_register = START_ADDRESS;
        assert!(cpu.execute(0xFC1E).is_ok());
        assert_eq!(cpu.index_register, START_ADDRESS + 0xB2);
    }

    #[test]
    fn op_fx29() {
        let mut cpu = CPU::new();
        cpu.v_register[0xC] = 0x4;
        assert!(cpu.execute(0xFC29).is_ok());
        assert_eq!(cpu.index_register, 20);
    }

    #[test]
    fn op_fx33() {
        let mut cpu = CPU::new();
        cpu.index_register = START_ADDRESS;
        cpu.v_register[0xC] = 123;
        assert!(cpu.execute(0xFC33).is_ok());
        assert_eq!(cpu.ram[(START_ADDRESS as usize)..=(START_ADDRESS as usize + 2)], [1, 2, 3]);
        cpu.v_register[0xC] = 205;
        assert!(cpu.execute(0xFC33).is_ok());
        assert_eq!(cpu.ram[(START_ADDRESS as usize)..=(START_ADDRESS as usize + 2)], [2, 0, 5]);
        cpu.v_register[0xC] = 002;
        assert!(cpu.execute(0xFC33).is_ok());
        assert_eq!(cpu.ram[(START_ADDRESS as usize)..=(START_ADDRESS as usize + 2)], [0, 0, 2]);
        cpu.v_register[0xC] = 140;
        assert!(cpu.execute(0xFC33).is_ok());
        assert_eq!(cpu.ram[(START_ADDRESS as usize)..=(START_ADDRESS as usize + 2)], [1, 4, 0]);
    }

    #[test]
    fn op_fx55() {
        let mut cpu = CPU::new();
        cpu.v_register = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        cpu.index_register = START_ADDRESS;
        let _ = cpu.execute(0xFC55);
        assert_eq!(cpu.ram[(START_ADDRESS as usize)..=(START_ADDRESS as usize + 0xC)], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
        cpu.index_register = RAM_SIZE as u16 - 5;
        let execute_result = cpu.execute(0xFC55);
        print!("{:?}", execute_result);
        assert!(execute_result.is_err());
    }

    #[test]
    fn op_fx65() {
        let mut cpu = CPU::new();
        for i in 0..=0xF {
            cpu.ram[START_ADDRESS as usize + i] = i as u8 + 1;
        }
        cpu.index_register = START_ADDRESS;
        let _ = cpu.execute(0xFC65);
        assert_eq!(cpu.v_register[0..=0xC], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
        cpu.index_register = RAM_SIZE as u16 - 5;
        let execute_result = cpu.execute(0xFC65);
        print!("{:?}", execute_result);
        assert!(execute_result.is_err());
    }
}
