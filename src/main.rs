use emulator::cartridge::Cartridge;
use std::fs;

use crate::emulator::cpu::CPU;

mod bytes;
mod emulator;

fn main() {
    let cart = Cartridge::new(
        fs::read("roms/The Legend of Zelda-Link's Awakening.gb")
            .expect("should be able to read a rom"),
    );
    let title: String = cart.header.title.iter().collect();
    println!("successfully loaded Rom: [{}]", title);
    let mut cpu = CPU::new(&cart);
    println!("CPU initialized");
    loop {
        cpu.execute_operation();
    }
}
