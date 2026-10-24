use std::{cmp::min, rc::Rc};

use softbuffer::Buffer;
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::application::font::get_display_bits;

pub struct CpuDisplay {
    width: u32,
    height: u32,
    screen: Vec<Vec<(u8, u8, u8)>>,

    title: String,
}

const BG_COLOR: (u8, u8, u8) = (0, 0, 0);
const TEXT_COLOR: (u8, u8, u8) = (128, 128, 128);

const SCALE_MULTIPLIER: u32 = 2;

impl CpuDisplay {
    pub fn new(title: String) -> CpuDisplay {
        CpuDisplay {
            width: 0,
            height: 0,
            screen: vec![],

            title: title,
        }
    }
    fn resize_screen(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.screen = vec![];
        for _ in 0..self.height {
            let mut row = vec![];
            for _ in 0..self.width {
                row.push(BG_COLOR);
            }
            self.screen.push(row);
        }
    }
    pub fn update_screen(&mut self) {
        self.draw_char(self.title.chars().nth(0).unwrap(), 10, 20);
    }
    fn draw_char(&mut self, character: char, x: u32, y: u32) {
        const CHAR_SIZE: u8 = 8;
        const HIGH_BYTE_MASK: u64 = 0xFF00_0000_0000_0000;
        const HIGH_BIT_MASK: u8 = 0x80;

        let mut char_bits = get_display_bits(&character);
        for y_offset in 0..CHAR_SIZE {
            let mut row_byte = ((char_bits & HIGH_BYTE_MASK) >> (64 - CHAR_SIZE)) as u8;
            for x_offset in 0..CHAR_SIZE {
                let bit = row_byte & HIGH_BIT_MASK;
                let color;
                if bit != 0 {
                    color = TEXT_COLOR;
                } else {
                    color = BG_COLOR;
                }
                self.screen[y.wrapping_add(y_offset as u32) as usize]
                    [x.wrapping_add(x_offset as u32) as usize] = color;
                row_byte = row_byte << 1;
            }
            char_bits = char_bits << CHAR_SIZE;
        }
    }
    pub fn draw_to_buffer(&mut self, buffer: &mut Buffer<OwnedDisplayHandle, Rc<Window>>) {
        if self.width != buffer.width().get() / SCALE_MULTIPLIER
            || self.height != buffer.height().get() / SCALE_MULTIPLIER
        {
            self.resize_screen(
                buffer.width().get() / SCALE_MULTIPLIER,
                buffer.height().get() / SCALE_MULTIPLIER,
            );
            self.update_screen();
        }

        for index in 0..(buffer.width().get() * buffer.height().get()) {
            let y = min((index / buffer.width().get()) / SCALE_MULTIPLIER, self.height - 1);
            let x = min ((index % buffer.width().get()) / SCALE_MULTIPLIER, self.width - 1);
            let pixel =
                self.screen[y as usize][x as usize];
            buffer[index as usize] =
                ((pixel.0 as u32) << 16) | ((pixel.1 as u32) << 8) | pixel.2 as u32
        }
    }
}
