use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::snes::emulator::emulator::SnesEmulator;

pub struct Application {
    pub window: Option<Rc<Window>>,
    pub surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    pub emulator: SnesEmulator,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("DimensionE")
                        .with_inner_size(LogicalSize::new(256.0, 224.0)),
                )
                .unwrap(),
        );
        let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
        self.surface = Some(softbuffer::Surface::new(&context, window.clone()).unwrap());
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.emulator.activate();
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
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
                    let mut y = (((index / buffer.width().get()) / 4) * 3) + 0x7FFD;
                    let mut x = (index % buffer.width().get()) / 4;
                    while x > 0xFF {
                        x -= 0xFF;
                    }
                    while y > 0xFFFF {
                        y -= 0xFFFF;
                    }
                    let red = self.emulator.memory_controller.read_byte(x as u8, y as u16);
                    let green = self
                        .emulator
                        .memory_controller
                        .read_byte(x as u8, (y as u16).wrapping_add(1));
                    let blue = self
                        .emulator
                        .memory_controller
                        .read_byte(x as u8, (y as u16).wrapping_add(2));

                    buffer[index as usize] =
                        blue as u32 | ((green as u32) << 8) | ((red as u32) << 16);
                }

                buffer.present().unwrap();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if matches!(event.physical_key, PhysicalKey::Code(KeyCode::Space))
                    && matches!(event.state, ElementState::Pressed)
                {
                    self.emulator.execute_cpu_op();
                }
                if matches!(event.physical_key, PhysicalKey::Code(KeyCode::Enter))
                    && matches!(event.state, ElementState::Pressed)
                {
                    self.emulator.should_run = !self.emulator.should_run;
                }
            }
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
