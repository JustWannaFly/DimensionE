use std::fs;

use crate::snes::emulator::emulator::SnesEmulator;
mod snes;

mod bytes;

use std::num::NonZeroU32;
use std::rc::Rc;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    emulator: SnesEmulator,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(event_loop.create_window(
            Window::default_attributes()
            .with_title("DimensionE")
            .with_inner_size(LogicalSize::new(256.0, 224.0))
        ).unwrap());
        let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
        self.surface = Some(softbuffer::Surface::new(&context, window.clone()).unwrap());
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // TODO implement per tick loop logic here
        // self.emulator.tick();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                let Some(surface) = &mut self.surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };
                let Some(window) = &self.window else {
                    eprintln!("RedrawRequested with window not set");
                    return;
                };
                let size = window.inner_size();
                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();
                let mut buffer = surface.buffer_mut().unwrap();
                for index in 0..(buffer.width().get() * buffer.height().get()) {
                    let bank_offset = 0;//(index % 0xFFFF) * 3;
                    let red = self.emulator.memory_controller.read_byte(bank_offset as u8 + 0, index as u16 % 0xFFFF);
                    let green = self.emulator.memory_controller.read_byte((bank_offset as u8).wrapping_add(1), index as u16 % 0xFFFF);
                    let blue = self.emulator.memory_controller.read_byte((bank_offset as u8).wrapping_add(2), index as u16 % 0xFFFF);

                    buffer[index as usize] = blue as u32 | ((green as u32) << 8) | ((red as u32) << 16);
                }

                buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App{
        surface: None,
        window: None,
        emulator: SnesEmulator::new(),
    };
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Zelda A Link to the Past.sfc").expect("should be able to read a rom"),
    );
    app.emulator.load_cartridge(&cart);
    let app_result = event_loop.run_app(&mut app);
    let Some(error) = app_result.err() else { println!("App exiting"); return () };
    dbg!(error);
}
