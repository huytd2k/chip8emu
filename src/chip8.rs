use crate::chip8::opcode::Opcode;

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
/**************************************/
mod opcode;

pub struct Chip8Interpreter {
    registers_v: [u8; 16],
    register_i: u16,
    delay_timer: u8,
    sound_timer: u8,
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
    /// Set v[x] = nn
    I6XNN(Opcode),
    /// Add nn to v[x] ~ v[x] += nn
    I7XNN(Opcode),
    /// Add nn to v[x] ~ v[x] += nn
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
        self.load_rom(path);
        loop {
            self.exec();
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
            Instruction::I1NNN(opcode) => {
                self.register_pc = opcode.nnn;
            }
            Instruction::I2NNN(opcode) => {
                self.stack.push(self.register_pc);
                self.register_pc = opcode.nnn;
            }
            Instruction::I3XNN(_) => {
                panic!("Not implemented");
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
        cpu.run_rom("bc_test.ch8");
    }
}
