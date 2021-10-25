extern crate minifb;
mod chip8;

use crate::chip8::Chip8Interpreter;
use minifb::{Key, Window, WindowOptions};
const FRAME_BUFFER_WIDTH: usize = 64;
const FRAME_BUFFER_HEIGHT: usize = 32;
fn main() {
    let mut window = Window::new(
        "Chip8 Emulator",
        FRAME_BUFFER_WIDTH*10,
        FRAME_BUFFER_HEIGHT*10,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut cpu = Chip8Interpreter::new(Some(&mut window));
    cpu.run_rom("ibmrom.ch8");
}