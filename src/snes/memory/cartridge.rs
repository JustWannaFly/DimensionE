use std::fmt::Debug;

use MapMode::{ExHiRom, HiRom, LoRom, SA1, SDD1, SPC7110};

pub struct Cartridge {
    bytes: Vec<u8>,
    pub header: RomHeader,
}
#[derive(Debug)]
pub struct RomHeader {
    pub _title: Vec<char>,
    pub speed: super::MemorySpeed,
    pub map_mode: MapMode,
    pub _chipset: u8,
    pub _rom_size: u8,
    pub _ram_size: u8,
    pub _country_code: u8,
    pub _developer_id: u8,
    pub _rom_version: u8,
    pub _checksum_compliment: u16,
    pub _checksum: u16,
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
        _title: header_data
            .get(0..21)
            .unwrap()
            .iter()
            .map(|it| *it as char)
            .collect(),
        speed: get_speed(header_data[21]),
        map_mode: get_map_mode(header_data[21]),
        _chipset: header_data[22],
        _rom_size: header_data[23],
        _ram_size: header_data[24],
        _country_code: header_data[25],
        _developer_id: header_data[26],
        _rom_version: header_data[27],
        _checksum_compliment: ((header_data[28] as u16) << 8) | header_data[29] as u16,
        _checksum: ((header_data[30] as u16) << 8) | header_data[31] as u16,
    }
}

impl Cartridge {
    pub fn new(bytes: Vec<u8>) -> Cartridge {
        Cartridge {
            header: build_header(&bytes),
            bytes,
        }
    }
    pub fn read_byte(&self, bank: u8, address: u16) -> Option<u8> {
        let location;
        match self.header.map_mode {
            LoRom => {
                location = lo_rom_lookup(bank, address);
            }
            HiRom => {
                location = hi_rom_lookup(bank, address);
            }
            _ => {
                dbg!(&self.header);
                todo!("Cartridge mapping not implemented");
            }
        }
        return self.bytes.get(location).copied();
    }
}

fn lo_rom_lookup(bank: u8, address: u16) -> usize {
    let bank_size = 0x8000_u16;
    let mapped_bank;
    let mapped_address;

    match bank {
        0x00..0x80 => mapped_bank = bank / 2,
        0x80..=0xFF => mapped_bank = (bank - 0x80) / 2,
    }
    if bank % 2 == 0 && address >= bank_size {
        mapped_address = address - bank_size;
    } else {
        mapped_address = address;
    }
    ((mapped_bank as usize) << 16) + mapped_address as usize
}

fn hi_rom_lookup(bank: u8, address: u16) -> usize {
    let bank_offset;
    match bank {
        0x40..=0x7D => bank_offset = bank - 0x40,
        0xFE..=0xFF => bank_offset = bank - 0xC0,
        _ => bank_offset = bank,
    }
    ((bank_offset as usize) << 16) + (address as usize)
}

#[cfg(test)]
mod cartridge_tests {
    use super::{hi_rom_lookup, lo_rom_lookup};

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

    #[test]
    fn hi_rom_lookup_test() {
        // banks $00-$3F
        assert_eq!(0x00_8000, hi_rom_lookup(0x00, 0x8000));
        assert_eq!(0x00_FFFF, hi_rom_lookup(0x00, 0xFFFF));
        assert_eq!(0x3F_8000, hi_rom_lookup(0x3F, 0x8000));
        assert_eq!(0x3F_FFFF, hi_rom_lookup(0x3F, 0xFFFF));
        // banks $40-$7D
        assert_eq!(0x00_0000, hi_rom_lookup(0x40, 0x0000));
        assert_eq!(0x00_FFFF, hi_rom_lookup(0x40, 0xFFFF));
        assert_eq!(0x3D_0000, hi_rom_lookup(0x7D, 0x0000));
        assert_eq!(0x3D_FFFF, hi_rom_lookup(0x7D, 0xFFFF));
        // banks $FE-$FF
        assert_eq!(0x3E_0000, hi_rom_lookup(0xFE, 0x0000));
        assert_eq!(0x3E_FFFF, hi_rom_lookup(0xFE, 0xFFFF));
        assert_eq!(0x3F_0000, hi_rom_lookup(0xFF, 0x0000));
        assert_eq!(0x3F_FFFF, hi_rom_lookup(0xFF, 0xFFFF));
    }
}
