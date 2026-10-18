use std::fs;

use crate::snes::emulator::emulator::SnesEmulator;
mod snes;

mod bytes;

use std::num::NonZeroU32;
use std::rc::Rc;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
        self.surface = Some(softbuffer::Surface::new(&context, window.clone()).unwrap());
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // TODO implement per tick loop logic here
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
                dbg!(window.scale_factor());
                let mut buffer = surface.buffer_mut().unwrap();
                for index in 0..(buffer.width().get() * buffer.height().get()) {
                    let y = index / buffer.width().get();
                    let x = index % buffer.width().get();
                    let red = x % 255;
                    let green = y % 255;
                    let blue = (x * y) % 255;

                    buffer[index as usize] = blue | (green << 8) | (red << 16);
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

    let mut app = App::default();
    let app_result = event_loop.run_app(&mut app);
    let Some(error) = app_result.err() else { println!("App exiting"); return () };
    dbg!(error);
}
fn run_emulator() {
    let cart = snes::memory::cartridge::Cartridge::new(
        fs::read("roms/Zelda A Link to the Past.sfc").expect("should be able to read a rom"),
    );
    let mut emulator = SnesEmulator::new();
    emulator.load_cartridge(&cart);
    let title_bytes = emulator.memory_controller.read(0, 0xFFC0, 21);
    println!(
        "Rom Load check:: Title: [{}]",
        bytes::build_string(&title_bytes)
    );
    loop {
        // TODO limit loop speed to master clock speed
        emulator.tick();
    }
}
