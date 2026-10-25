use softbuffer::Surface;
use std::alloc::System;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::application::cpu_display::CpuDisplay;
use crate::snes::emulator::emulator::SnesEmulator;

mod cpu_display;
mod font;

const DEBUG_WIDTH: u32 = 600;
const DEBUG_HEIGHT: u32 = 400;

const FRAME_DRAW_DELAY: u128 = 1000 / 60; // (millis / target) FPS

pub struct Application {
    display_window: Option<Rc<Window>>,
    display_window_id: Option<WindowId>,
    display_surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,

    debug_window: Option<Rc<Window>>,
    debug_window_id: Option<WindowId>,
    debug_surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    debug_display: CpuDisplay,

    last_draw: Instant,

    pub emulator: SnesEmulator,
}

impl Application {
    pub fn new() -> Application {
        Application {
            display_surface: None,
            display_window: None,
            display_window_id: None,
            debug_window: None,
            debug_window_id: None,
            debug_surface: None,
            debug_display: CpuDisplay::new(),

            last_draw: Instant::now(),

            emulator: SnesEmulator::new(),
        }
    }
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
        self.display_surface = Some(softbuffer::Surface::new(&context, window.clone()).unwrap());
        self.display_window_id = Some(window.id());
        self.display_window = Some(window);
        self.display_window.as_ref().unwrap().request_redraw();

        let debug_window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("DimensionE CPU")
                        .with_inner_size(LogicalSize::new(DEBUG_WIDTH, DEBUG_HEIGHT)),
                )
                .unwrap(),
        );
        let debug_context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
        self.debug_surface =
            Some(softbuffer::Surface::new(&debug_context, debug_window.clone()).unwrap());
        self.debug_window_id = Some(debug_window.id());
        self.debug_window = Some(debug_window);
        self.debug_window.as_ref().unwrap().request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.emulator.activate();

        // TODO fix timing of all activations this is crude and not desired behavior
        let draw_duration = self.last_draw.elapsed().as_millis();
        if draw_duration >= FRAME_DRAW_DELAY {
            self.display_window.as_ref().unwrap().request_redraw();
            self.debug_window.as_ref().unwrap().request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.last_draw = Instant::now();
                if id == self.display_window_id.unwrap() {
                    self.draw_display_window();
                }
                if id == self.debug_window_id.unwrap() {
                    self.draw_debug_window();
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if matches!(event.physical_key, PhysicalKey::Code(KeyCode::ArrowRight))
                    && matches!(event.state, ElementState::Pressed)
                {
                    self.emulator.cycle_cpu();
                }
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

impl Application {
    fn draw_display_window(&mut self) {
        let Some(surface) = &mut self.display_surface else {
            eprintln!("RedrawRequested fired before Resumed or after Suspended");
            return;
        };
        let Some(window) = &self.display_window else {
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
        let mut display_buffer = surface.buffer_mut().unwrap();
        for index in 0..(display_buffer.width().get() * display_buffer.height().get()) {
            let mut y = (((index / display_buffer.width().get()) / 4) * 3) + 0x7FFD;
            let mut x = (index % display_buffer.width().get()) / 4;
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

            display_buffer[index as usize] =
                blue as u32 | ((green as u32) << 8) | ((red as u32) << 16);
        }

        display_buffer.present().unwrap();
    }
    fn draw_debug_window(&mut self) {
        let Some(debug_surface) = &mut self.debug_surface else {
            eprintln!("RedrawRequested fired before Resumed or after Suspended");
            return;
        };
        let Some(debug_window) = &self.debug_window else {
            eprintln!("Trying to redraw not initialized debug_window");
            return;
        };
        debug_surface
            .resize(
                NonZeroU32::new(debug_window.inner_size().width).unwrap(),
                NonZeroU32::new(debug_window.inner_size().height).unwrap(),
            )
            .unwrap();
        let mut debug_buffer = debug_surface.buffer_mut().unwrap();
        self.debug_display.draw_to_buffer(&mut debug_buffer, &self.emulator.get_cart_title(), &self.emulator.cpu);
        debug_buffer.present().unwrap();
    }
}
