use crate::chip8::Chip8Interpreter;

mod chip8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = &args[1];

    let mut interpreter = Chip8Interpreter::new();
    interpreter.run_rom(rom_path);
}