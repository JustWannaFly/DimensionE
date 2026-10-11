use crate::{bytes::split_bytes, snes::memory::cartridge::Cartridge};

pub mod cartridge;

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
    cart_loaded: bool,
    cart_speed: u8,
    memory_data: Vec<Vec<u8>>,
    pub reset_vector: u16,
    cpu_bus: Bus,
}

/// Currently mirrored data is simply stored multiple times rather than aliased.
impl MemoryController {
    pub fn new() -> MemoryController {
        MemoryController {
            cart_loaded: false,
            cart_speed: 0,
            memory_data: vec![vec![0; 0xFFFF + 1]; 0xFF + 1],
            reset_vector: 0x0000,
            cpu_bus: Bus {
                bank: 0x00,
                address: 0x0000,
            },
        }
    }
    pub fn load_cart(&mut self, cartridge: &Cartridge) {
        self.cart_speed = match cartridge.header.speed {
            MemorySpeed::Fast => 6,
            MemorySpeed::Slow => 8,
        };
        self.reset_vector = ((cartridge.read_byte(0x00, 0xFFFD).unwrap() as u16) << 8)
            + cartridge.read_byte(0x00, 0xFFFC).unwrap() as u16;

        for bank in 0x00..=0x3F as u8 {
            for address in 0x8000..=0xFFFF as u16 {
                match cartridge.read_byte(bank, address) {
                    Some(byte) => self.memory_data[bank as usize][address as usize] = byte,
                    _ => (),
                }
                cartridge.read_byte(bank, address);
            }
        }

        self.cart_loaded = true;
    }
    pub fn _unload_cart(&mut self) {
        self.cart_loaded = false;
    }
    pub fn cycles_to_access(&self, bank: u8, address: u16) -> u8 {
        if !self.cart_loaded {
            return 0;
        }
        match bank {
            0x00..0x40 => match address {
                0x0000..0x2000 => 8,
                0x2000..0x4000 => 6,
                0x4000..0x4200 => 12,
                0x4200..0x6000 => 6,
                0x6000..=0xFFFF => 8,
            },
            0x40..0x80 => 8,
            0x80..0xC0 => match address {
                0x0000..0x2000 => 8,
                0x2000..0x4000 => 6,
                0x4000..0x4200 => 12,
                0x4200..0x6000 => 6,
                0x6000..0x8000 => 8,
                0x8000..=0xFFFF => self.cart_speed,
            },
            0xC0..=0xFF => self.cart_speed,
        }
    }
    pub fn read(&self, bank: u8, address: u16, length: u16) -> Vec<u8> {
        // TODO implement timing logic
        if !self.cart_loaded {
            return vec![0];
        }
        let bank = self.memory_data.get(bank as usize).unwrap();
        let mut result = vec![];
        let mut read_addr = address;
        while result.len() < length as usize {
            result.push(*bank.get(read_addr as usize).unwrap());
            read_addr = read_addr.wrapping_add(1);
        }
        result
    }
    pub fn read_byte(&self, bank: u8, address: u16) -> u8 {
        if !self.cart_loaded {
            return 0;
        }
        let bank = self.memory_data.get(bank as usize).unwrap();
        *(&bank[address as usize])
    }
    pub fn write(&mut self, bank: u8, address: u16, bytes: Vec<u8>) {
        // TODO implement timing logic
        let mut index = address;
        bytes.iter().for_each(|byte| {
            self.memory_data[bank as usize][index as usize] = *byte;
            self.write_mirrors(bank, index, *byte);
            index = index.wrapping_add(1);
        });
    }
    fn write_mirrors(&mut self, bank: u8, address: u16, byte: u8) {
        if (bank <= 0x3F && address <= 0x1FFF)
            || (bank == 0x7E && address <= 0x1FFF)
            || (0x80 <= bank && bank <= 0xBF && address <= 0x1FFF)
        {
            for bank in 0x00..=0x3F {
                self.memory_data[bank][address as usize] = byte;
            }
            self.memory_data[0x7E as usize][address as usize] = byte;
            for bank in 0x80..=0xBF {
                self.memory_data[bank][address as usize] = byte;
            }
        }
    }
    /// Sets the cpu bus to a given location and returns the number 
    /// of master cycles needed to access that location in memory
    pub fn cpu_bus_set_location(&mut self, bank: u8, address: u16) -> u8 {
        self.cpu_bus.bank = bank;
        self.cpu_bus.address = address;
        return self.cycles_to_access(bank, address);
    }
    pub fn cpu_bus_read8(&self) -> u8 {
        self.read_bus(&self.cpu_bus)
    }
    pub fn cpu_bus_write16(&mut self, value: u16) {
        let bytes = split_bytes(value);
        self.write(self.cpu_bus.bank, self.cpu_bus.address, vec![bytes.0, bytes.1]);
    }
    fn read_bus(&self, bus: &Bus) -> u8 {
        if !self.cart_loaded {
            return 0;
        }
        let bank = self.memory_data.get(bus.bank as usize).unwrap();
        *(&bank[bus.address as usize])
    }
    pub fn write_bus(&mut self, bus: &Bus, byte: u8) {
        self.memory_data[bus.bank as usize][bus.address as usize] = byte;
        self.write_mirrors(bus.bank, bus.address, byte);
    }
}
