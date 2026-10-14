use std::fs;

use crate::snes::emulator::emulator::SnesEmulator;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Chrono Trigger.sfc")
            .expect("should be able to read a rom"),
    );
    let mut emulator = SnesEmulator::new();
    emulator.load_cartridge(&cart);
    let title_bytes = emulator.memory_controller.read(0, 0xFFC0, 21);
    println!("Rom Load check:: Title: [{}]", bytes::build_string(&title_bytes));
    loop { // TODO limit loop speed to master clock speed
        emulator.tick();
    }

}


