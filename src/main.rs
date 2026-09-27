use std::fs;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/zelda.sfc")
            .expect("should be able to read a rom"),
    );
    let title: String = cart.header.title.iter().collect();
    println!("successfully loaded Rom: [{}]", title);
    dbg!(cart.header);
    
}
