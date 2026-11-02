use std::fs;

use crate::application::Application;
mod snes;

mod bytes;

use winit::event_loop::{ControlFlow, EventLoop};

mod application;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new();
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Super Mario World.sfc").expect("should be able to read a rom"),
    );
    app.emulator.load_cartridge(&cart);
    let app_result = event_loop.run_app(&mut app);
    let Some(error) = app_result.err() else {
        println!("App exiting");
        return ();
    };
    dbg!(error);
}
