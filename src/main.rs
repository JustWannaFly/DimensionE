use std::fs;

use eframe::{EframeWinitApplication, egui, egui_wgpu::winit};

use crate::{bytes::build_string, snes::emulator::emulator::SnesEmulator};
mod snes;

mod bytes;
fn main() {

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("DimensionE Debug Window", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
    
    // let cart = snes::memory::cartridge::Cartridge::new(
    //     fs::read("roms/Zelda A Link to the Past.sfc")
    //         .expect("should be able to read a rom"),
    // );
    // let mut emulator = SnesEmulator::new();
    // emulator.load_cartridge(&cart);
    // let title_bytes = emulator.memory_controller.read(0, 0xFFC0, 21);
    // println!("Rom Load check:: Title: [{}]", bytes::build_string(&title_bytes));
    // loop { // TODO limit loop speed to master clock speed
    //     emulator.tick();
    // }

}


struct MyEguiApp {
    emulator: SnesEmulator,
    rom_title: String,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Zelda A Link to the Past.sfc")
            .expect("should be able to read a rom"),
    );
    let mut emulator = SnesEmulator::new();
    emulator.load_cartridge(&cart);

    let title = build_string(&emulator.memory_controller.read(0, 0xFFC0, 21));
        MyEguiApp {
            emulator: emulator,
            rom_title: title,
        }

    }
}

impl eframe::App for MyEguiApp {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ui, |ui| {
           ui.heading(&self.rom_title);
       });
   }
   fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       self.emulator.tick();
   }
}
