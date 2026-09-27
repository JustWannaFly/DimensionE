use std::fs;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/zelda.sfc")
            .expect("should be able to read a rom"),
    );
    let title: String = cart.header.title.iter().collect();
    let read_title: Vec<char> = cart.read(0x80, 0xFFC0, 21).unwrap().iter().map(|byte| {*byte as char}).collect();
    dbg!(read_title);
    println!("successfully loaded Rom: [{}]", title);
    
}
