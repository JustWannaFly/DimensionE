pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

pub fn split_bytes(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, val as u8)
}

pub fn is_carry16(val1: u16, val2: u16) -> bool {
    val1 as u32 + val2 as u32 > 0xFFFF
}

pub fn is_carry8(val1: u8, val2: u8) -> bool {
    val1 as u16 + val2 as u16 > 0xFF
}

pub fn is_half_carry16(val1: u16, val2: u16) -> bool {
    let half_carry_bitmask: u16 = 0b0000_0111_1111_1111;
    val1 & half_carry_bitmask + val2 & half_carry_bitmask > half_carry_bitmask
}

pub fn is_half_carry8(val1: u8, val2: u8) -> bool {
    let half_carry_bitmask: u8 = 0b0111;
    val1 & half_carry_bitmask + val2 & half_carry_bitmask > half_carry_bitmask
}

pub fn build_flag_byte(
    curr_flags: u8,
    z: Option<bool>,
    n: Option<bool>,
    h: Option<bool>,
    c: Option<bool>,
) -> u8 {
    let mut maintain_mask: u8 = 0x0;
    let mut set_mask: u8 = 0x0;
    match z {
        Some(is_set) => {
            if is_set {
                set_mask += 0b1000_0000
            }
        }
        None => maintain_mask += 0b1000_0000,
    }
    match n {
        Some(is_set) => {
            if is_set {
                set_mask += 0b0100_0000
            }
        }
        None => maintain_mask += 0b0100_0000,
    }
    match h {
        Some(is_set) => {
            if is_set {
                set_mask += 0b0010_0000
            }
        }
        None => maintain_mask += 0b0010_0000,
    }
    match c {
        Some(is_set) => {
            if is_set {
                set_mask += 0b0001_0000
            }
        }
        None => maintain_mask += 0b0001_0000,
    }
    (curr_flags & maintain_mask) | set_mask
}
