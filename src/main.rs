use std::fs;

use crate::snes::emulator::emulator::SnesEmulator;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Zelda A Link to the Past.sfc")
            .expect("should be able to read a rom"),
    );
    let mut emulator = SnesEmulator::new();
    emulator.memory_controller.load_cart(&cart);
    let title_bytes = emulator.memory_controller.read(0, 0xFFC0, 21);
    println!("Rom Load check:: Title: [{}]", build_string(&title_bytes));

}

fn build_string(bytes: &Vec<u8>) -> String {
    bytes.iter().map(|f| {*f as char}).collect()
}
