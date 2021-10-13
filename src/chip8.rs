pub struct Chip8Interpreter {
    registers_v: [u16; 16],
    registers_i: [u16; 2],
    delay_timer: u8,
    sound_timer: u8,
    register_pc: u16,
    mem: Mem,
    pixels: [[u8; 64]; 32],
}

#[derive(Debug)]
enum Instruction {
    End(u16),
    Clear(u16),
    Jump(u16),
    Sevx(u16),
    Advx(u16),
    Seri(u16),
    Draw(u16),
}

type Mem = [u8; 4096];

impl Chip8Interpreter {
    pub fn new() -> Chip8Interpreter {
        let mut mem = [0; 4096];
        let fonts = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        for x in 0..fonts.len() {
            mem[x] = fonts[x];
        }
        Chip8Interpreter {
            registers_v: [0; 16],
            registers_i: [0; 2],
            delay_timer: 0,
            sound_timer: 0,
            register_pc: 0x200,
            pixels: [[0; 64]; 32],
            mem,
        }
    }

    fn load_rom(&mut self, path: &str) {
        let file = std::fs::read(path).unwrap();
        if file.len() > 4096 {
            panic!("File too long!!");
        }
        for (idx, &byte) in file.iter().enumerate() {
            self.mem[0x200+idx] = byte;
        }
    }

    pub fn run_rom(&mut self, path: &str) {
        self.load_rom(path);
        loop {
            self.exec();
        }
    }

    fn exec(&mut self) {
        let opcode = self.fetch(self.register_pc);
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn fetch(&mut self, addr: u16) -> u16{
        let addr = addr as usize;
        self.register_pc += 2;
        ((self.mem[addr] as u16) << 8) + (self.mem[addr+1] as u16) 
    }

    fn display(&self) {
        for x in self.pixels {
            println!("{}", x.map(|x| if x > 0 {"■"} else {" "}).join(""))
        }
    }

    fn take_mem_at_vi(&self) -> u8 {
        self.mem[self.registers_i[0] as usize]
    }
    
    fn decode(&self, opcode: u16) -> Instruction{
        if opcode == 0x00 {
            // std::process::exit(0);
            return Instruction::End(opcode);
        }
        if opcode == 0x00E0 {
            return Instruction::Clear(opcode);
        }
        if opcode >> 12 == 0x1 {
            return Instruction::Jump(opcode);
        }
        if opcode >> 12 == 0x6 {
            return Instruction::Sevx(opcode);
        }
        if opcode >> 12 == 0x7 {
            return Instruction::Advx(opcode);
        }
        if opcode >> 12 == 0xA {
            return Instruction::Seri(opcode);
        }
        if opcode >> 12 == 0xD {
            return Instruction::Draw(opcode);
        }
        panic!("Unknow opcode {:#04x} at pc {}!", opcode, self.register_pc);
    }
    
    fn execute(&mut self, inst: Instruction) {
        match inst {
            Instruction::End(_) => {
                std::process::exit(0);
            }
            Instruction::Clear(opcode) => {
                self.pixels = [[0; 64]; 32];
            }
            Instruction::Jump(opcode) => {
                self.register_pc = take_param_nnn(opcode);
            }
            Instruction::Sevx(opcode) => {
                let x = take_param_x(opcode);
                let kk = take_param_kk(opcode);
                self.registers_v[x as usize] = kk as u16;
            }
            Instruction::Advx(opcode) => {
                let x = take_param_x(opcode);
                let kk = take_param_kk(opcode);
                self.registers_v[x as usize] += kk as u16;
            }
            Instruction::Seri(opcode) => {
                let nnn = take_param_nnn(opcode);
                self.registers_i[0] = nnn;
            }
            Instruction::Draw(opcode) => {
                let x = take_param_x(opcode);
                let y = take_param_y(opcode);
                let n = take_param_n(opcode);
                let x_cor = self.registers_v[x as usize] & 63;
                let y_cor = self.registers_v[y as usize] & 31;
                self.registers_v[0xF] = 0;
                self.registers_v[0xF] = display(&mut self.pixels, self.mem, self.registers_i[0], x_cor, y_cor, n as u16);
                // self.display();
            }
        }
    }
}

fn take_param_n(opcode: u16) -> u8 {
    (opcode & 0x000F) as u8
}

fn take_param_nnn(opcode: u16) -> u16 {
    opcode & 0x0FFF
}

fn take_param_kk(opcode: u16) -> u8 {
    (opcode & 0x00FF) as u8
}

fn take_param_x(opcode: u16) -> u8 {
    ((opcode & 0x0F00) >> 8) as u8
}

fn take_param_y(opcode: u16) -> u8 {
    ((opcode & 0x00F0) >> 4) as u8
}

fn display(pixels: &mut [[u8; 64]; 32], mem: Mem, i: u16, x_cor: u16, y_cor: u16, n: u16) -> u16 {
    let mut ret = 0;
    for row in 0..n {
        let mut sprite = mem[(i + row) as usize];
        for x in 0..8{
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
    fn test_take_param_n() {
        assert_eq!(take_param_n(0x8C74), 0x04);
        assert_eq!(take_param_n(0x8B74), 0x04);
    }

    #[test]
    fn test_take_param_nnn() {
        assert_eq!(take_param_nnn(0x8C74), 0x0C74);
        assert_eq!(take_param_nnn(0xAB12), 0x0B12);
    }

    #[test]
    fn test_take_param_kk() {
        assert_eq!(take_param_kk(0x8C74), 0x74);
        assert_eq!(take_param_kk(0xAB12), 0x12);
    }

    #[test]
    fn test_take_param_x() {
        assert_eq!(take_param_x(0x8C74), 0x0C);
        assert_eq!(take_param_x(0xAB12), 0x0B);
    }

    #[test]
    fn test_take_param_y() {
        assert_eq!(take_param_y(0x8C74), 0x07);
        assert_eq!(take_param_y(0xAB12), 0x01);
    }

    #[test]
    fn test_cpu_fetch() {
        let mut cpu = Chip8Interpreter::new();
        cpu.mem[0] = 0xAB;
        cpu.mem[1] = 0xBC;
        assert_eq!(cpu.fetch(0), 0xABBC);
        assert_eq!(cpu.register_pc, 2);
    }

    #[test]
    fn test_cpu_load() {
        let mut cpu = Chip8Interpreter::new();
        cpu.load_rom("tests/resource/0xABBC.txt");
        assert_eq!(cpu.fetch(0), 0xABBC);
    }

    #[test]
    fn test_display() {
        let mut cpu = Chip8Interpreter::new();
        cpu.mem[0] = 0b11111000;
        cpu.mem[1] = 0;
        cpu.pixels = [[1; 64]; 32];
        assert_eq!(display(&mut cpu.pixels, cpu.mem, 0, 63, 31, 1), 1);
        assert_eq!(display(&mut cpu.pixels, cpu.mem, 1, 63, 31, 1), 0);
        assert_eq!(cpu.pixels[31][63], 0);
        assert_eq!(cpu.pixels[31][0], 0);
        assert_eq!(cpu.pixels[31][1], 0);
        assert_eq!(cpu.pixels[31][2], 0);
        assert_eq!(cpu.pixels[31][3], 0);
    }
}
