use std::fmt::Debug;

use MapMode::{ExHiRom, HiRom, LoRom, SA1, SDD1, SPC7110};

pub struct Cartridge {
    bytes: Vec<u8>,
    pub header: RomHeader,
}
#[derive(Debug)]
pub struct RomHeader {
    pub title: Vec<char>,
    pub speed: super::MemorySpeed,
    pub map_mode: MapMode,
    pub chipset: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub country_code: u8,
    pub developer_id: u8,
    pub rom_version: u8,
    pub checksum_compliment: u16,
    pub checksum: u16,
}

fn get_speed(header_byte: u8) -> super::MemorySpeed {
    if header_byte & 0b0001_0000 == 0 {
        super::MemorySpeed::Slow
    } else {
        super::MemorySpeed::Fast
    }
}

#[derive(Debug)]
pub enum MapMode {
    LoRom,
    HiRom,
    SDD1,
    SA1,
    ExHiRom,
    SPC7110,
}
fn get_map_mode(header_byte: u8) -> MapMode {
    match header_byte & 0b0000_1111 {
        0 => LoRom,
        1 => HiRom,
        2 => SDD1,
        3 => SA1,
        5 => ExHiRom,
        0xA => SPC7110,
        _ => panic!("Mapping mode did not match known mapping"),
    }
}

fn is_valid_title(title_chars: &[u8]) -> bool {
    let mut is_valid = true;
    let title_char_low: u8 = 0x20;
    let title_char_high: u8 = 0x7E;
    title_chars.iter().for_each(|char| {
        if title_char_low > *char || *char > title_char_high {
            is_valid = false
        }
    });
    is_valid
}

// TODO add handling/detection for roms with a copy tool header added
fn build_header(raw_data: &Vec<u8>) -> RomHeader {
    let low_rom_header_start = 0x7FC0;
    let high_rom_header_start = 0xFFC0;
    let ex_high_rom_header_start = 0x40FFC0;
    let header_length = 32;
    // TODO verify only 3 possible header locations
    let low_header = raw_data.get(low_rom_header_start..low_rom_header_start + header_length);
    let high_header = raw_data.get(high_rom_header_start..high_rom_header_start + header_length);
    let ex_high_header =
        raw_data.get(ex_high_rom_header_start..ex_high_rom_header_start + header_length);
    let header_data: &[u8];
    let low_flag = match low_header {
        Some(header_data) => header_data[21],
        None => 0,
    };
    let high_flag = match high_header {
        Some(header_data) => header_data[21],
        None => 0,
    };
    if (low_flag == 0b0010_0000 || low_flag == 0b0011_0000)
        && is_valid_title(
            low_header
                .unwrap()
                .get(0..21)
                .expect("expected low_header title chars to exist"),
        )
    {
        header_data = low_header.unwrap()
    } else if (high_flag == 0b0010_0001 || high_flag == 0b0011_0001)
        && is_valid_title(
            high_header
                .unwrap()
                .get(0..21)
                .expect("expected high_header title chars to exist"),
        )
    {
        header_data = high_header.unwrap();
    } else {
        header_data = ex_high_header.unwrap()
    }
    RomHeader {
        title: header_data
            .get(0..21)
            .unwrap()
            .iter()
            .map(|it| *it as char)
            .collect(),
        speed: get_speed(header_data[21]),
        map_mode: get_map_mode(header_data[21]),
        chipset: header_data[22],
        rom_size: header_data[23],
        ram_size: header_data[24],
        country_code: header_data[25],
        developer_id: header_data[26],
        rom_version: header_data[27],
        checksum_compliment: ((header_data[28] as u16) << 8) | header_data[29] as u16,
        checksum: ((header_data[30] as u16) << 8) | header_data[31] as u16,
    }
}

impl Cartridge {
    pub fn new(bytes: Vec<u8>) -> Cartridge {
        Cartridge {
            header: build_header(&bytes),
            bytes,
        }
    }
    pub fn _read(&self, bank: u8, start_address: u16, end_address: u16) -> &[u8] {
        let start_location: usize;
        let end_location: usize;
        match self.header.map_mode {
            LoRom => {
                start_location = lo_rom_lookup(bank, start_address);
                end_location = lo_rom_lookup(bank, end_address);
            }
            _ => {
                dbg!(&self.header);
                panic!("Cartridge mapping not implemented");
            }
        }
        self.bytes.get(start_location..=end_location).expect("Rom data should be here")
    }
    pub fn read_byte(&self, bank: u8, address: u16) -> Option<u8> {
       return self.bytes.get(lo_rom_lookup(bank, address)).copied()
    }
}

fn lo_rom_lookup(bank: u8, address: u16) -> usize {
    let bank_size: usize = 0x8000;
    let bank_to_access = if bank < 0x80 { bank } else { bank - 0x80 };
    let bank_offset = bank_to_access as usize * bank_size;

    let mut address_offset = address as usize;
    if address_offset >= bank_size {
        address_offset -= bank_size;
    }
    bank_offset + address_offset
}

#[cfg(test)]
mod cartridge_tests {
    use super::lo_rom_lookup;

    #[test]
    fn lo_rom_lookup_test() {
        // banks $00-$3F
        assert_eq!(0x00_0000, lo_rom_lookup(0x00, 0x8000));
        assert_eq!(0x00_7FFF, lo_rom_lookup(0x00, 0xFFFF));
        assert_eq!(0x1F_8000, lo_rom_lookup(0x3F, 0x8000));
        assert_eq!(0x1F_FFFF, lo_rom_lookup(0x3F, 0xFFFF));
        // banks $40-$6F
        assert_eq!(0x20_0000, lo_rom_lookup(0x40, 0x0000));
        assert_eq!(0x20_7FFF, lo_rom_lookup(0x40, 0x7FFF));
        assert_eq!(0x37_8000, lo_rom_lookup(0x6F, 0x0000));
        assert_eq!(0x37_FFFF, lo_rom_lookup(0x6F, 0x7FFF));

        assert_eq!(0x20_0000, lo_rom_lookup(0x40, 0x8000));
        assert_eq!(0x20_7FFF, lo_rom_lookup(0x40, 0xFFFF));
        assert_eq!(0x37_8000, lo_rom_lookup(0x6F, 0x8000));
        assert_eq!(0x37_FFFF, lo_rom_lookup(0x6F, 0xFFFF));
        // banks $70-$7D
        assert_eq!(0x38_0000, lo_rom_lookup(0x70, 0x8000));
        assert_eq!(0x38_7FFF, lo_rom_lookup(0x70, 0xFFFF));
        assert_eq!(0x3E_8000, lo_rom_lookup(0x7D, 0x8000));
        assert_eq!(0x3E_FFFF, lo_rom_lookup(0x7D, 0xFFFF));
        // banks $FE-$FF
        assert_eq!(0x3F_0000, lo_rom_lookup(0xFE, 0x8000));
        assert_eq!(0x3F_7FFF, lo_rom_lookup(0xFE, 0xFFFF));
        assert_eq!(0x3F_8000, lo_rom_lookup(0xFF, 0x8000));
        assert_eq!(0x3F_FFFF, lo_rom_lookup(0xFF, 0xFFFF));
    }
}
