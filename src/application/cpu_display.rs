use std::{cmp::min, rc::Rc};

use softbuffer::Buffer;
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::{application::font::get_display_bits, bytes::{get_byte_as_hex, get_word_as_hex}, snes::cpu::CPU};

pub struct CpuDisplay {
    width: u32,
    height: u32,
    screen: Vec<Vec<(u8, u8, u8)>>,
}

const BG_COLOR: (u8, u8, u8) = (0, 0, 0);
const TEXT_COLOR: (u8, u8, u8) = (128, 128, 128);
const CHAR_SIZE: u8 = 8;

const SCALE_MULTIPLIER: u32 = 2;


impl CpuDisplay {
    pub fn new() -> CpuDisplay {
        CpuDisplay {
            width: 0,
            height: 0,
            screen: vec![],
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
    pub fn update_screen(&mut self, title: &str, cpu: &CPU) {
        const COL_1: u32 = 85;
        const COL_2: u32 = 210;
        self.draw_string(title, 50, 10);
        self.draw_string(&get_word_as_hex(cpu.accumulator), COL_1, 25);
        self.draw_string(&get_word_as_hex(cpu.x), COL_1, 40);
        self.draw_string(&get_word_as_hex(cpu.y), COL_1, 55);
        self.draw_string(&get_word_as_hex(cpu.direct_page), COL_1, 70);
        self.draw_string(&get_word_as_hex(cpu.stack_pointer), COL_1, 85);
        self.draw_string(&get_byte_as_hex(cpu.processor_status), COL_1, 100);
        self.draw_string(&get_byte_as_hex(cpu.data_bank), COL_1, 115);
        self.draw_string(&get_byte_as_hex(cpu.program_bank), COL_1, 130);
        self.draw_string(&get_word_as_hex(cpu.program_counter), COL_1, 145);

        let data = &cpu.processing_data;
        let operand = match data.operand {
            Some(value) => get_word_as_hex(value),
            None => "None".to_string(),
        };

        self.draw_string(&format!("{:?} ", cpu.processing_state), COL_2, 25);
        self.draw_string(&format!("{:?}    ", data.op), COL_2, 40); // TODO fix trailing characters not being overwritten
        self.draw_string(&get_byte_as_hex(data.args.len() as u8), COL_2, 55);
        self.draw_string(&operand, COL_2, 70);
        self.draw_string(&get_byte_as_hex(data.op_cycles), COL_2, 85);
        self.draw_string(&get_byte_as_hex(data.write_count), COL_2, 100);
        self.draw_string(&get_byte_as_hex(data.read_count), COL_2, 115);
    }
    fn draw_static_labels(&mut self) {
        const COL_1: u32 = 10;
        const COL_2: u32 = 135;
        self.draw_char('[', 40, 10);
        self.draw_char(']', 220, 10);
        self.draw_string("accum   :", COL_1, 25);
        self.draw_string("x reg   :", COL_1, 40);
        self.draw_string("y reg   :", COL_1, 55);
        self.draw_string("dir  pg :", COL_1, 70);
        self.draw_string("stk ptr :", COL_1, 85);
        self.draw_string("status  :", COL_1, 100);
        self.draw_string("data bnk:", COL_1, 115);
        self.draw_string("pgrm bnk:", COL_1, 130);
        self.draw_string("pgrm ctr:", COL_1, 145);

        self.draw_string("state   :", COL_2, 25);
        self.draw_string("opertion:", COL_2, 40);
        self.draw_string("arg cnt :", COL_2, 55);
        self.draw_string("operand :", COL_2, 70);
        self.draw_string("cycle ct:", COL_2, 85);
        self.draw_string("write ct:", COL_2, 100);
        self.draw_string("read ct :", COL_2, 115);
    }
    fn draw_string(&mut self, string: &str, x: u32, y: u32) {
        let mut x_offset = 0_u32;
        for character in string.chars() {
            self.draw_char(character, x + x_offset, y);
            x_offset += CHAR_SIZE as u32;
        }
    }
    fn draw_char(&mut self, character: char, x: u32, y: u32) {
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
                let y_index = y + y_offset as u32;
                let x_index = x + x_offset as u32;
                if y_index >= self.height || x_index >= self.width {
                    return;
                }
                self.screen[y_index as usize][x_index as usize] = color;
                row_byte = row_byte << 1;
            }
            char_bits = char_bits << CHAR_SIZE;
        }
    }
    pub fn draw_to_buffer(&mut self, buffer: &mut Buffer<OwnedDisplayHandle, Rc<Window>>, title: &str, cpu: &CPU) {
        if self.width != buffer.width().get() / SCALE_MULTIPLIER
            || self.height != buffer.height().get() / SCALE_MULTIPLIER
        {
            self.resize_screen(
                buffer.width().get() / SCALE_MULTIPLIER,
                buffer.height().get() / SCALE_MULTIPLIER,
            );
            self.draw_static_labels();
        }
        self.update_screen(title, cpu);

        for index in 0..(buffer.width().get() * buffer.height().get()) {
            let y = min(
                (index / buffer.width().get()) / SCALE_MULTIPLIER,
                self.height - 1,
            );
            let x = min(
                (index % buffer.width().get()) / SCALE_MULTIPLIER,
                self.width - 1,
            );
            let pixel = self.screen[y as usize][x as usize];
            buffer[index as usize] =
                ((pixel.0 as u32) << 16) | ((pixel.1 as u32) << 8) | pixel.2 as u32
        }
    }
}
