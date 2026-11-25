use std::{cell::RefCell, rc::Rc};

use crate::{
    bytes::{combine_bytes, split_bytes},
    snes::apu::spc700::Spc700,
};

mod spc700;

struct AramController {
    aram: Vec<u8>,
}

impl AramController {
    pub fn new() -> AramController {
        AramController {
            aram: vec![0; 0xFFFF + 1],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        self.aram[address as usize]
    }
}

pub struct Apu {
    io_registers: [u16; 4],
    aram_controller: Rc<RefCell<AramController>>,
    cpu: Spc700,
}

impl Apu {
    pub fn new() -> Apu {
        let aram_controller = Rc::new(RefCell::new(AramController::new()));
        Apu {
            io_registers: [0, 0, 0, 0],
            cpu: Spc700::new(aram_controller.clone()),
            aram_controller: aram_controller,
        }
    }
    pub fn write_cpu_io(&mut self, register: u8, byte: u8) {
        assert!(register < 4);
        let existing_bytes = split_bytes(self.io_registers[register as usize]);
        let updated_bytes = combine_bytes(existing_bytes.1, byte);
        self.io_registers[register as usize] = updated_bytes;
    }
    pub fn read_cpu_io(&self, register: u8) -> u8 {
        assert!(register < 4);
        split_bytes(self.io_registers[register as usize]).1
    }
    fn write_apu_io(&mut self, register: u8, byte: u8) {
        assert!(register < 4);
        let existing_bytes = split_bytes(self.io_registers[register as usize]);
        let updated_bytes = combine_bytes(byte, existing_bytes.0);
        self.io_registers[register as usize] = updated_bytes;
    }
    fn read_apu_io(&self, register: u8) -> u8 {
        assert!(register < 4);
        split_bytes(self.io_registers[register as usize]).0
    }
    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}
