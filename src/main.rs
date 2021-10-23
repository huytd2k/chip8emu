extern crate minifb;
mod chip8;

use crate::chip8::Chip8Interpreter;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut cpu = Chip8Interpreter::new();
    cpu.run_rom("ibmrom.ch8");
}