use std::fs;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Zelda A Link to the Past.sfc")
            .expect("should be able to read a rom"),
    );
    println!("starting memory load...");
    let mut memory_controller = snes::memory::MemoryController::new();
    memory_controller.load_cart(&cart);
    println!("Cartridge loading complete");
    let mem_title: String = memory_controller.read(0, 0xFFC0, 21).iter().map(|f| {*f as char}).collect();
    println!("Rom Title: [{}]", mem_title);
}
