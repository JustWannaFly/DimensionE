use std::fs;
mod snes;

mod bytes;
fn main() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Chrono Trigger.sfc")
            .expect("should be able to read a rom"),
    );
    println!("starting memory load...");
    let mut memory_controller = snes::memory::MemoryController::new();
    memory_controller.load_cart(&cart);
    println!("Cartridge loading complete");
    let title_bytes = memory_controller.read(0, 0xFFC0, 21);
    println!("Rom Load check:: Title: [{}]", build_string(&title_bytes));

    memory_controller.write(0x01, 0x0000, title_bytes);
    let read_write_title_bytes = memory_controller.read(0x01, 0x00_0000, 21);
    println!("Memory Read & Write check:: Title: [{}]", build_string(&read_write_title_bytes));
}

fn build_string(bytes: &Vec<u8>) -> String {
    bytes.iter().map(|f| {*f as char}).collect()
}
