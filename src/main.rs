use std::fs;
use emulator::cartridge::Cartridge;

mod emulator;
mod bytes;

fn main() {
    let cart =
        Cartridge::new(fs::read("roms/The Legend of Zelda-Link's Awakening.gb").expect("should be able to read a rom"));
    dbg!(&cart.header);
    let title: String = cart.header.title.iter().collect();
    println!("successfully loaded Rom: [{}]", title)
}

