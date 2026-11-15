use std::{cell::RefCell, rc::Rc};

use crate::{bytes::combine_bytes, snes::{apu::Apu, memory::cartridge::Cartridge}};

pub mod cartridge;

const FAST_MEMORY_CYCLES: u8 = 6;
const SLOW_MEMORY_CYCLES: u8 = 8;
const XSLOW_MEMORY_CYCLES: u8 = 12;

#[derive(Debug, Clone, Copy)]
pub enum MemorySpeed {
    Fast,
    Slow,
}

struct Bus {
    bank: u8,
    address: u16,
}
pub struct MemoryController {
    apu: Rc<RefCell<Apu>>,
    cartridge: Option<Cartridge>,
    cart_speed: u8,
    memory_data: Vec<Vec<u8>>,
    pub reset_vector: u16,
    cpu_bus: Bus,
}

/// Currently mirrored data is simply stored multiple times rather than aliased.
impl MemoryController {
    pub fn new(apu: Rc<RefCell<Apu>>) -> MemoryController {
        MemoryController {
            apu,
            cartridge: None,
            cart_speed: 0,
            // TODO trim the size down to not hold rom or mirrored data
            memory_data: vec![vec![0; 0xFFFF + 1]; 0x80 + 1],
            reset_vector: 0x0000,
            cpu_bus: Bus {
                bank: 0x00,
                address: 0x0000,
            },
        }
    }
    pub fn load_cart(&mut self, cartridge: Cartridge) {
        self.cart_speed = match cartridge.header.speed {
            MemorySpeed::Fast => FAST_MEMORY_CYCLES,
            MemorySpeed::Slow => SLOW_MEMORY_CYCLES,
        };
        self.reset_vector = combine_bytes(
            cartridge.read_byte(0x00, 0xFFFD).unwrap(),
            cartridge.read_byte(0x00, 0xFFFC).unwrap(),
        );
        self.cartridge = Some(cartridge);
    }
    pub fn cycles_to_access(&self, bank: u8, address: u16) -> u8 {
        match bank {
            0x00..0x40 => match address {
                0x0000..0x2000 => SLOW_MEMORY_CYCLES,
                0x2000..0x4000 => FAST_MEMORY_CYCLES,
                0x4000..0x4200 => XSLOW_MEMORY_CYCLES,
                0x4200..0x6000 => FAST_MEMORY_CYCLES,
                0x6000..=0xFFFF => SLOW_MEMORY_CYCLES,
            },
            0x40..0x80 => SLOW_MEMORY_CYCLES,
            0x80..0xC0 => match address {
                0x0000..0x2000 => SLOW_MEMORY_CYCLES,
                0x2000..0x4000 => FAST_MEMORY_CYCLES,
                0x4000..0x4200 => XSLOW_MEMORY_CYCLES,
                0x4200..0x6000 => FAST_MEMORY_CYCLES,
                0x6000..0x8000 => SLOW_MEMORY_CYCLES,
                0x8000..=0xFFFF => self.cart_speed,
            },
            0xC0..=0xFF => self.cart_speed,
        }
    }
    pub fn read(&self, bank: u8, address: u16, length: u16) -> Vec<u8> {
        let mut result = vec![];
        let mut read_addr = address;
        while result.len() < length as usize {
            result.push(self.read_byte(bank, read_addr));
            read_addr = read_addr.wrapping_add(1);
        }
        result
    }
    pub fn read_byte(&self, bank: u8, address: u16) -> u8 {
        match bank {
            0x00..0x40 => match address {
                0x0000..0x2140 => self.memory_data[0x00][address as usize],
                0x2140..0x2180 => self.read_apu_io(address),
                0x2180..0x8000 => self.memory_data[0x00][address as usize],
                0x8000..=0xFFFF => self.read_cartridge(bank, address),
            },
            0x40..0x7E => self.read_cartridge(bank, address),
            0x7E => match address {
                0x0000..0x2000 => self.memory_data[0x00][address as usize],
                0x2000..=0xFFFF => self.memory_data[bank as usize][address as usize],
            },
            0x7F => self.memory_data[bank as usize][address as usize],
            0x80..0xFE => self.read_byte(bank - 0x80, address),
            0xFE..=0xFF => self.read_cartridge(bank, address),
        }
    }
    fn read_cartridge(&self, bank: u8, address: u16) -> u8 {
        if let Some(cartridge) = &self.cartridge {
            cartridge.read_byte(bank, address).unwrap_or(0)
        } else { 0 }
    }
    fn write_apu_io(&self, address: u16, byte: u8) {
        let apu_base_address = 0x2140_u16;
        let mut register = address - apu_base_address;
        while register >= 4 {
            register -= 4;
        }
        self.apu.borrow_mut().write_cpu_io(register as u8, byte);
    }
    fn read_apu_io(&self, address: u16) -> u8 {
        let apu_base_address = 0x2140_u16;
        let mut register = address - apu_base_address;
        while register >= 4 {
            register -= 4;
        }
        self.apu.borrow_mut().read_cpu_io(register as u8)
    }
    pub fn write(&mut self, bank: u8, address: u16, byte: u8) {
        match bank {
            0x00..0x40 => match address {
                0x0000..0x2140 => self.memory_data[0x00][address as usize] = byte,
                0x2140..0x2180 => self.write_apu_io(address, byte),
                0x2180..0x8000 => self.memory_data[0x00][address as usize] = byte,
                0x8000..=0xFFFF => (),
            },
            0x40..0x7E => (),
            0x7E => match address {
                0x0000..0x2000 => self.memory_data[0x00][address as usize] = byte,
                0x2000..=0xFFFF => self.memory_data[bank as usize][address as usize] = byte,
            },
            0x7F => self.memory_data[bank as usize][address as usize] = byte,
            0x80..0xFE => self.write(bank - 0x80, address, byte),
            0xFE..=0xFF => (),
        };
    }
    /// Sets the cpu bus to a given location and returns the number
    /// of master cycles needed to access that location in memory
    pub fn cpu_bus_set_location(&mut self, bank: u8, address: u16) -> u8 {
        self.cpu_bus.bank = bank;
        self.cpu_bus.address = address;
        return self.cycles_to_access(bank, address);
    }
    pub fn cpu_bus_read(&self) -> u8 {
        self.read_byte(self.cpu_bus.bank, self.cpu_bus.address)
    }
    pub fn cpu_bus_write(&mut self, value: u8) {
        self.write(self.cpu_bus.bank, self.cpu_bus.address, value);
    }
}
