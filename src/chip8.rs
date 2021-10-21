extern crate crossbeam_channel;
use std::time::Duration;
use crate::chip8::opcode::Opcode;
use crossbeam_channel::{tick, select};

// Declare specification in constant
const MEMORY_SIZE: u16 = 4096;
// In Chip-8, the memory from address 0x00 -> 0x199 is preserved
const FIRST_LOADABLE_ADDR: u16 = 0x200;
const FRAME_BUFFER_WIDTH: usize = 64;
const FRAME_BUFFER_HEIGHT: usize = 32;
const FONTS_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const INSTRUCTIONS_PER_SECOND: f64 = 700.;
/**************************************/
mod opcode;

pub struct Chip8Interpreter {
    registers_v: [u8; 16],
    register_i: u16,
    delay_timer: u16,
    sound_timer: u16,
    register_pc: u16,
    mem: Mem,
    frame_buffer: [[u8; FRAME_BUFFER_WIDTH]; FRAME_BUFFER_HEIGHT],
    stack: Vec<u16>,
}


#[derive(Debug)]
enum Instruction {
    End(Opcode),
    /// Clear screen
    I00E0(Opcode),
    /// Return subroutine. i.e: pc = stack.pop()
    I00EE(Opcode),

    /// Jump to instruction ~ pc = nnn
    I1NNN(Opcode),

    /// Create subroutine ~ stack.push(pc) & pc = nnn
    I2NNN(Opcode),

    /// Skip next instruction if x = nn
    I3XNN(Opcode),

    /// Skip next instruction if x != nn
    I4XNN(Opcode),

    /// Skip next instruction if x == y
    I5XY0(Opcode),

    /// Set v[x] = nn
    I6XNN(Opcode),

    /// Add nn to v[x] ~ v[x] += nn
    I7XNN(Opcode),

    /// Set v[x] = v[y]
    I8XY0(Opcode),

    /// v[x] = v[x] OR v[y]
    I8XY1(Opcode), 

    /// v[x] = v[x] AND v[y]
    I8XY2(Opcode), 

    /// v[x] = v[x] XOR v[y]
    I8XY3(Opcode), 

    /// v[x] = v[x] + v[y], set v[f] = 1 if overflow
    I8XY4(Opcode), 

    /// v[x] = v[x] - v[y], set v[f] = 0 if underflow
    I8XY5(Opcode), 

    /// v[x] = v[y] - v[x], set v[f] = 0 if underflow
    I8XY7(Opcode), 

    /// if old_shift: v[x] = v[y], v[x] >> 1 
    I8XY6(Opcode), 

    /// Skip next instruction if x != y
    I9XY0(Opcode),

    /// Set vi = nnn
    IANNN(Opcode),

    /// Draw
    IDXYN(Opcode),
}


type Mem = [u8; MEMORY_SIZE as usize];

fn init_mem() -> Mem {
    let mut mem = [0; 4096];
    // Load font into memory
    for x in 0..FONTS_DATA.len() {
        mem[x] = FONTS_DATA[x];
    }
    
    mem
}
impl Chip8Interpreter {
    pub fn new() -> Chip8Interpreter {
        Chip8Interpreter {
            registers_v: [0; 16],
            register_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            register_pc: FIRST_LOADABLE_ADDR,
            frame_buffer: [[0; FRAME_BUFFER_WIDTH]; FRAME_BUFFER_HEIGHT],
            stack: vec![],
            mem: init_mem(),
        }
    }

    fn load_rom(&mut self, path: &str) {
        let file = std::fs::read(path).unwrap();
        if file.len() > 4096 - FIRST_LOADABLE_ADDR as usize {
            panic!("File too long!!");
        }
        for (idx, &byte) in file.iter().enumerate() {
            self.mem[0x200 + idx] = byte;
        }
    }

    pub fn run_rom(&mut self, path: &str) {
        let timer_ticker = tick(Duration::from_millis(((1.0/60.0)*1000.) as u64));
        let cpu_timer = tick(Duration::from_millis(((1.0/INSTRUCTIONS_PER_SECOND)*1000.) as u64));
        self.load_rom(path);
        loop {
            select! {
                recv(timer_ticker) -> _ => {
                    if self.delay_timer != 0 {
                        self.delay_timer -= 1;
                    }
                    if self.sound_timer != 0 {
                        self.sound_timer -= 1;
                    }
                }
                recv(cpu_timer) -> _ => {
                    if self.delay_timer == 0 {
                        self.exec();
                    }
                }
            }
        }
    }

    fn exec(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn fetch(&mut self) -> u16 {
        let addr = self.register_pc as usize;
        self.register_pc += 2;
        ((self.mem[addr] as u16) << 8) | (self.mem[addr + 1] as u16)
    }

    fn display(&self) {
        for x in self.frame_buffer {
            println!("{}", x.map(|x| if x > 0 { "■" } else { " " }).join(""))
        }
    }

    fn decode(&self, raw_opcode: u16) -> Instruction {
        let opcode = Opcode::new(raw_opcode);
        if raw_opcode == 0x00 {
            return Instruction::End(opcode);
        }
        if raw_opcode == 0x00E0 {
            return Instruction::I00E0(opcode);
        }
        if raw_opcode == 0x00EE {
            return Instruction::I00EE(opcode);
        }
        if raw_opcode >> 12 == 0x1 {
            return Instruction::I1NNN(opcode);
        }
        if raw_opcode >> 12 == 0x2 {
            return Instruction::I2NNN(opcode);
        }
        if raw_opcode >> 12 == 0x3 {
            return Instruction::I3XNN(opcode);
        }
        if raw_opcode >> 12 == 0x4 {
            return Instruction::I4XNN(opcode);
        }
        if raw_opcode >> 12 == 0x5 {
            return Instruction::I5XY0(opcode);
        }
        if raw_opcode >> 12 == 0x6 {
            return Instruction::I6XNN(opcode);
        }
        if raw_opcode >> 12 == 0x7 {
            return Instruction::I7XNN(opcode);
        }
        if raw_opcode >> 12 == 0xA {
            return Instruction::IANNN(opcode);
        }
        if raw_opcode >> 12 == 0x9 {
            return Instruction::I9XY0(opcode);
        }
        if raw_opcode >> 12 == 0xD {
            return Instruction::IDXYN(opcode);
        }
        panic!("Unknow opcode {:#04x} at pc {}!", raw_opcode, self.register_pc);
    }
    fn execute(&mut self, inst: Instruction) {
        match inst {
            Instruction::End(_) => {
                std::process::exit(0);
            }
            Instruction::I00E0(_) => {
                self.frame_buffer = [[0; FRAME_BUFFER_WIDTH]; FRAME_BUFFER_HEIGHT];
            }   
            Instruction::I00EE(_) => {
                // NOTE: error handling
                self.register_pc = self.stack.pop().unwrap();
            }
            Instruction::I1NNN(opcode) => {
                self.register_pc = opcode.nnn;
            }
            Instruction::I2NNN(opcode) => {
                self.stack.push(self.register_pc);
                self.register_pc = opcode.nnn;
            }
            Instruction::I3XNN(opcode) => {
                if opcode.x == opcode.kk {
                    self.register_pc += 2;
                }
            }
            Instruction::I4XNN(opcode) => {
                if opcode.x != opcode.kk {
                    self.register_pc += 2;
                }
            }
            Instruction::I5XY0(opcode) => {
                if opcode.x == opcode.y {
                    self.register_pc += 2;
                }
            }
            Instruction::I9XY0(opcode) => {
                if opcode.x != opcode.y {
                    self.register_pc += 2;
                }
            }
            Instruction::I6XNN(opcode) => {
                self.registers_v[opcode.x as usize] = opcode.kk;
            }
            Instruction::I7XNN(opcode) => {
                self.registers_v[opcode.x as usize] += opcode.kk;
            }
            Instruction::IANNN(opcode) => {
                self.register_i = opcode.nnn;
            }
            Instruction::IDXYN(opcode) => {
                let x_cor = self.registers_v[opcode.x as usize] & 63;
                let y_cor = self.registers_v[opcode.y as usize] & 31;
                self.registers_v[0xF] = 0;
                self.registers_v[0xF] =
                    display(&mut self.frame_buffer, self.mem, self.register_i, x_cor, y_cor, opcode.n);
            }
            _ => panic!("Instruction not implemented!"),
        }
    }
}

fn display(pixels: &mut [[u8; FRAME_BUFFER_WIDTH]; FRAME_BUFFER_HEIGHT], mem: Mem, i: u16, x_cor: u8, y_cor: u8, n: u8) -> u8 {
    let mut ret = 0;
    for row in 0..n {
        let mut sprite = mem[(i + row as u16) as usize];
        for x in 0..8 {
            if sprite >> 7 > 0 {
                let to_y = ((y_cor + row) & 31) as usize;
                let to_x = ((x_cor + x) & 63) as usize;
                let old = pixels[to_y][to_x];
                pixels[to_y][to_x] ^= 1;
                if old != pixels[to_y][to_x] {
                    ret = 1;
                }
            }
            sprite <<= 1;
        }
    }

    ret
}

fn shift_left_carry(val: &mut u8) -> u8 {
    let shifted_out = *val >> 7;
    *val <<= 1;

    shifted_out
}

fn shift_right_carry(val: &mut u8) -> u8 {
    let shifted_out = *val & 0x1;
    *val >>= 1;

    shifted_out
}

fn add_carry(a: u8, b: u8) -> (u8, u8) {
    let sum_16 = (a as u16) + (b as u16);

    ((sum_16 >> 8) as u8, (sum_16 & 0xFF) as u8)
}

fn subtract_carry(a: u8, b: u8) -> (u8, u8) {
    if a >= b {
        (0, a-b)
    } else {
        (1, (256+(a as u16) - (b as u16)) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_fetch() {
        let mut cpu = Chip8Interpreter::new();
        cpu.mem[0x200] = 0xAB;
        cpu.mem[0x201] = 0xBC;
        assert_eq!(cpu.fetch(), 0xABBC);
        assert_eq!(cpu.register_pc, 0x202);
    }

    #[test]
    fn test_cpu_load() {
        let mut cpu = Chip8Interpreter::new();
        cpu.load_rom("tests/resource/0xABBC.txt");
        assert_eq!(cpu.fetch(), 0xABBC);
    }

    #[test]
    fn test_display() {
        let mut cpu = Chip8Interpreter::new();
        cpu.mem[0] = 0b11111000;
        cpu.mem[1] = 0;
        cpu.frame_buffer = [[1; FRAME_BUFFER_WIDTH]; FRAME_BUFFER_HEIGHT];
        assert_eq!(display(&mut cpu.frame_buffer, cpu.mem, 0, 63, 31, 1), 1);
        assert_eq!(display(&mut cpu.frame_buffer, cpu.mem, 1, 63, 31, 1), 0);
        assert_eq!(cpu.frame_buffer[31][63], 0);
        assert_eq!(cpu.frame_buffer[31][0], 0);
        assert_eq!(cpu.frame_buffer[31][1], 0);
        assert_eq!(cpu.frame_buffer[31][2], 0);
        assert_eq!(cpu.frame_buffer[31][3], 0);
    }

    #[test]
    #[ignore]
    fn test_bc() {
        let mut cpu = Chip8Interpreter::new();
        // cpu.delay_timer = 60;
        cpu.run_rom("ibmrom.ch8");
    }

    #[test]
    fn test_shift_left() {
        let mut val = 0b1011_1111;
        assert_eq!(shift_left_carry(&mut val), 1);
        assert_eq!(val, 0b0111_1110);

        let mut val = 0b0011_1111;
        assert_eq!(shift_left_carry(&mut val), 0);
        assert_eq!(val, 0b0111_1110);
    }

    #[test]
    fn test_shift_right() {
        let mut val = 0b1011_1111;
        assert_eq!(shift_right_carry(&mut val), 1);
        assert_eq!(val, 0b0101_1111);

        let mut val = 0b0011_1110;
        assert_eq!(shift_right_carry(&mut val), 0);
        assert_eq!(val, 0b0001_1111);
    }

    #[test]
    fn test_add_carry() {
        assert_eq!(add_carry(0, 0), (0, 0));
        assert_eq!(add_carry(1, 1), (0, 2));
        assert_eq!(add_carry(5, 13), (0, 18));
        assert_eq!(add_carry(255, 1), (1, 0));
        assert_eq!(add_carry(255, 10), (1, 9));
        assert_eq!(add_carry(255, 255), (1, 254));
    }

    #[test]
    fn test_subtract_carry() {
        assert_eq!(subtract_carry(0, 0), (0, 0));
        assert_eq!(subtract_carry(0, 1), (1, 255));
        assert_eq!(subtract_carry(10, 20), (1, 246));
    }
}
