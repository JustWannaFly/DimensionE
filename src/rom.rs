use std::{fmt::Debug, fs::TryLockError::Error, iter::Map};

use crate::rom::MapMode::{ExHiRom, HiRom, LoRom, SA1, SDD1, SPC7110};

pub struct Rom {
    bytes: Vec<u8>,
}
#[derive(Debug)]
pub struct RomHeader {
    pub title: Vec<char>,
    pub speed: Speed,
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

#[derive(Debug)]
pub enum Speed {
    Slow,
    Fast
}
impl Speed {
    pub fn get_speed(header_byte: u8) -> Speed {
        if header_byte & 0b0001_0000 == 0 {Speed::Slow} else {Speed::Fast}
    }
}

#[derive(Debug)]
pub enum MapMode {
    LoRom,
    HiRom,
    SDD1,
    SA1,
    ExHiRom,
    SPC7110
}
impl MapMode {
    pub fn get_map_mode(header_byte: u8) -> MapMode {
        match header_byte & 0b0000_1111 {
            0 => LoRom,
            1 => HiRom,
            2 => SDD1,
            3 => SA1,
            5 => ExHiRom,
            0xA => SPC7110,
            _ => panic!("Mapping mode did not match known mapping")
        }
    }
}

impl Rom {
    pub fn new(bytes: Vec<u8>) -> Rom {
        Rom { bytes }
    }
    pub fn read(&self, address: usize, length: usize) -> Option<&[u8]> {
        self.bytes.get(address..(address + length))
    }
    pub fn get_header(&self) -> RomHeader {
        let low_rom_header_start = 0x7FC0;
        let high_rom_header_start = 0xFFC0;
        let ex_high_rom_header_start = 0x40FFC0;
        let header_length = 32;
        dbg!(header_length);
        let low_header = self.read(low_rom_header_start, header_length);
        let high_header = self.read(high_rom_header_start, header_length);
        let ex_high_header = self.read(ex_high_rom_header_start, header_length);
        let header_data: &[u8];
        let low_flag = match low_header {
            Some(header_data) => header_data[21],
            None => 0
        };
        let high_flag = match high_header {
            Some(header_data) => header_data[21],
            None => 0
        };
        let ex_high_flag = match ex_high_header {
            Some(header_data) => header_data[21],
            None => 0
        };
        dbg!(low_flag);
        dbg!(high_flag);
        dbg!(ex_high_flag);
        if low_flag == 0b0010_0000 || low_flag == 0b0011_0000 {
            header_data = low_header.unwrap()
        } else if high_flag == 0b0010_0001 || high_flag == 0b0011_0001 {
            header_data = high_header.unwrap();
        } else {
            header_data = ex_high_header.unwrap()
        }
        RomHeader {
            title: header_data.get(0..21).unwrap().iter().map(|it| *it as char).collect(),
            speed: Speed::get_speed(header_data[21]),
            map_mode: MapMode::get_map_mode(header_data[21]),
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
}
