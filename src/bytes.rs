pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

pub fn split_bytes(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, val as u8)
}
