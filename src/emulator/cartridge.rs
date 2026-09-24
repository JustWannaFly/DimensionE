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
    pub gbc_flag: u8,
    pub new_liscensee_code: u16,
    pub sgb_flag: u8,
    pub cartridge_type: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub dest_code: u8,
    pub old_liscensee_code: u8,
    pub rom_version: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
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
            gbc_flag: header_data[0x0143],
            new_liscensee_code: combine_bytes(header_data[0x0144], header_data[0x0145]),
            sgb_flag: header_data[0x0146],
            cartridge_type: header_data[0x0147],
            rom_size: header_data[0x0148],
            ram_size: header_data[0x0149],
            dest_code: header_data[0x014A],
            old_liscensee_code: header_data[0x014B],
            rom_version: header_data[0x014C],
            header_checksum: header_data[0x014D],
            global_checksum: combine_bytes(header_data[0x014E], header_data[0x014F]),
        }
    }
}
