use std::fs;
use rom::Rom;

mod rom;

fn main() {
    let rom =
        Rom::new(fs::read("roms/zelda.sfc").expect("should be able to read a rom"));
    let rom_header = rom.get_header();
    dbg!(&rom_header);
    let title: String = rom_header.title.iter().collect();
    println!("successfully loaded Rom: [{}]", title)
}

