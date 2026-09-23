use emulator::cartridge::Cartridge;
use std::fs;

mod bytes;
mod emulator;

fn main() {
    let cart = Cartridge::new(
        fs::read("roms/The Legend of Zelda-Link's Awakening.gb")
            .expect("should be able to read a rom"),
    );
    dbg!(&cart.header);
    let title: String = cart.header.title.iter().collect();
    println!("successfully loaded Rom: [{}]", title)
}
