/// combine_bytes(high: 0x12, low: 0x34) -> 0x1234
pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((low as u16) << 8) | high as u16
}

/// split_bytes(val: 0x1234) -> (0x34, 0x12)
pub fn split_bytes(val: u16) -> (u8, u8) {
    (val as u8, (val >> 8) as u8)
}
