pub fn _combine_bytes(high: u8, low: u8) -> u16 {
    ((low as u16) << 8) | high as u16
}

pub fn _split_bytes(val: u16) -> (u8, u8) {
    (val as u8, (val >> 8) as u8)
}
