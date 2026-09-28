use crate::snes::memory::cartridge::Cartridge;

pub mod cartridge;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum MemorySpeed {
    Fast,
    Slow,
    _XSlow,
}

pub struct MemoryController {
    cart_loaded: bool,
    cart_speed: MemorySpeed,
    memory_data: Vec<Vec<u8>>,
}

/// Currently mirrored data is simply stored multiple times rather than aliased.
impl MemoryController {
    pub fn new() -> MemoryController {
        MemoryController {
            cart_loaded: false,
            cart_speed: MemorySpeed::Slow,
            memory_data: vec![vec![0; 0xFFFF + 1]; 0xFF + 1],
        }
    }
    pub fn load_cart(&mut self, cartridge: &Cartridge) {
        // TODO write all the cartridge data into memory_data
        self.cart_speed = cartridge.header.speed;

        // TODO there is definitely a better way to do this. If loading a rom is slow, this is probably it.
        for bank in 0x00..=0x3F as u8 {
            for address in 0x8000..=0xFFFF as u16 {
                println!("loading {:x}:{:x}", bank, address);
                self.memory_data[bank as usize][address as usize] = cartridge.read_byte(bank, address);
            }
        }

        self.cart_loaded = true;
    }
    pub fn unload_cart(&mut self) {
        
        self.cart_loaded = false;
    }
    pub fn _read(&mut self, bank: u8, address: u16, length: u8) -> Vec<u8> {
        // TODO implement timing logic
        let bank = self.memory_data.get(bank as usize).unwrap();
        let mut result = vec![];
        let mut read_addr = address;
        while result.len() < length as usize {
            result.push(*bank.get(read_addr as usize).unwrap());
            read_addr = read_addr.wrapping_add(1);
        }
        result
    }
    pub fn write(&mut self, bank: u8, address: u16, bytes: Vec<u8>) {
        // TODO implement timing logic
        let mut index = address;
        bytes.iter().for_each(|byte| {
            self.memory_data[bank as usize][index as usize] = *byte;
            self.write_mirrors(bank, index, byte);
            index = index.wrapping_add(1);
        });
    }
    fn write_mirrors(&mut self, bank: u8, address: u16, byte: &u8) {
        if (bank <= 0x3F && address <= 0x1FFF)
        || (bank == 0x7E && address <= 0x1FFF)
        || (0x80 <= bank && bank <= 0xBF && address <= 0x1FFF) {
            for bank in 0x00..=0x3F {
                self.memory_data[bank][address as usize] = *byte;
            }
            self.memory_data[0x7E as usize][address as usize] = *byte;
            for bank in 0x80..=0xBF {
                self.memory_data[bank][address as usize] = *byte;
            }
        }
    }
}
