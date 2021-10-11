pub struct Chip8Interpreter {
    registers_v: [u16; 16],
    registers_i: [u16; 2],
    delay_timer: u8,
    sound_timer: u8,
    register_pc: u16,
    mem: Mem,
    pixels: [[u8; 64]; 32],
}

enum Instruction {
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
        Chip8Interpreter {
            registers_v: [0; 16],
            registers_i: [0; 2],
            delay_timer: 0,
            sound_timer: 0,
            register_pc: 0,
            mem: [0; 4096],
            pixels: [[0; 64]; 32],
        }
    }

    fn load_rom(&mut self, path: &str) {
        let file = std::fs::read(path).unwrap();
        if file.len() > 4096 {
            panic!("File too long!!");
        }
        for (idx, &byte) in file.iter().enumerate() {
            self.mem[idx] = byte;
        }
    }

    pub fn run_rom(&mut self, path: &str) {
        self.load_rom(path);
        loop {
            self.exec();
        }
    }

    fn exec(&mut self) {
        panic!("Not implemented!");
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
    
    fn decode(opcode: u16) -> Instruction{
        if opcode == 0x00E0 {
            return Instruction::Clear(opcode);
        }
        if opcode >> 12 == 0x1 {
            return Instruction::Jump(opcode);
        }
        if opcode >> 12 == 0x6 {
            return Instruction::Sevx(opcode);
        }
        if opcode >> 12 == 0xA {
            return Instruction::Seri(opcode);
        }
        if opcode >> 12 == 0xD {
            return Instruction::Seri(opcode);
        }
        panic!("Unknow opcode!")
    }
    
    fn execute(&mut self, inst: Instruction) {
        match inst {
            Instruction::Clear(opcode) => {
                self.pixels = [[0; 64]; 32];
            }
            Instruction::Jump(opcode) => {
                self.register_pc = take_param_nnn(opcode)
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
                let sprite_tall = self.take_mem_at_vi();
                let x = take_param_x(opcode);
                let y = take_param_y(opcode);
                let n = take_param_n(opcode);
                let x_cor = self.registers_v[x as usize] & 63;
                let y_cor = self.registers_v[y as usize] & 31;
                self.registers_v[15] = 0;
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
    // #[ignore]
    fn test_cpu_display() {
        let mut cpu = Chip8Interpreter::new();
        cpu.pixels = [[1; 64]; 32];
        cpu.pixels[1][1] = 0;
        cpu.display();
    }
}
