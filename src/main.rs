use std::{fs, str::FromStr};

fn main() {
    let rom =
        Rom::new(fs::read("roms/ChronoTrigger.sfc").expect("should be able to read a rom"));
    let rom_header = rom.get_header();
    let title: String = rom_header.title.iter().collect();
    println!("loaded Rom: [{}]", title)
}

struct Rom {
    bytes: Vec<u8>,
    // header: RomHeader,
}
struct RomHeader {
    title: Vec<char>,
    speed: u8,
    chipset: u8,
    rom_size: u16,
    ram_size: u16,
    country_code: u8,
    developer_id: u8,
    rom_version: u8,
    checksum_compliment: u16,
    checksum: u16,
}

impl Rom {
    pub fn new(bytes: Vec<u8>) -> Rom {
        Rom { bytes }
    }
    pub fn read(&self, address: usize, length: usize) -> Option<&[u8]> {
        self.bytes.get(address..(address + length))
    }
    pub fn get_header(&self) -> RomHeader {
        let header_addr = 0xFFC0;
        let mut low_ram_offset = header_addr - 0x7FC0;
        low_ram_offset = 0;
        let header_start = header_addr - low_ram_offset;
        let header_length = 0xFFDF - header_addr;
        let raw = self.bytes.get(header_start..(header_start + header_length))
            .expect("Should have header data in rom");
        dbg!(header_start);
        RomHeader {
            title: raw.get(0..21).unwrap().iter().map(|it| *it as char).collect(),
            speed: 0x0,
            chipset: 0x0,
            rom_size: 0x0,
            ram_size: 0x0,
            country_code: 0x0,
            developer_id: 0x0,
            rom_version: 0x0,
            checksum_compliment: 0x0,
            checksum: 0x0,
        }
    }
}
