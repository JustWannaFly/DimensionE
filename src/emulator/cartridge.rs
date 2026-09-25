use crate::bytes::combine_bytes;

pub struct Cartridge {
    data: Vec<u8>,
    pub header: CartridgeHeader,
}
impl Cartridge {
    pub fn new(rom_data: Vec<u8>) -> Cartridge {
        Cartridge {
            header: CartridgeHeader::new(&rom_data),
            data: rom_data,
        }
    }
    pub fn read8(&self, addr: usize) -> u8 {
        self.data[addr]
    }
    pub fn read16(&self, addr: usize) -> u16 {
        combine_bytes(self.data[addr], self.data[addr + 1])
    }
}

#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: [char; 15],
    pub _gbc_flag: u8,
    pub _new_liscensee_code: u16,
    pub _sgb_flag: u8,
    pub _cartridge_type: u8,
    pub _rom_size: u8,
    pub _ram_size: u8,
    pub _dest_code: u8,
    pub _old_liscensee_code: u8,
    pub _rom_version: u8,
    pub _header_checksum: u8,
    pub _global_checksum: u16,
}
impl CartridgeHeader {
    pub fn new(header_data: &[u8]) -> CartridgeHeader {
        CartridgeHeader {
            title: header_data
                .get(0x0134..=0x0142)
                .unwrap()
                .iter()
                .map(|it| *it as char)
                .collect::<Vec<char>>()
                .try_into()
                .unwrap(),
            _gbc_flag: header_data[0x0143],
            _new_liscensee_code: combine_bytes(header_data[0x0144], header_data[0x0145]),
            _sgb_flag: header_data[0x0146],
            _cartridge_type: header_data[0x0147],
            _rom_size: header_data[0x0148],
            _ram_size: header_data[0x0149],
            _dest_code: header_data[0x014A],
            _old_liscensee_code: header_data[0x014B],
            _rom_version: header_data[0x014C],
            _header_checksum: header_data[0x014D],
            _global_checksum: combine_bytes(header_data[0x014E], header_data[0x014F]),
        }
    }
}
