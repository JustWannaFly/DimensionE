/// combine_bytes(high: 0x12, low: 0x34) -> 0x1234
pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

/// split_bytes(val: 0x1234) -> (0x34, 0x12)
pub fn split_bytes(val: u16) -> (u8, u8) {
    (val as u8, (val >> 8) as u8)
}

/// checks if all the set bits in the mask are set in the value
pub fn check_bits_set(value: u8, mask: u8) -> bool {
    value & mask == mask
}
#[cfg(test)]
pub fn check_bits_unset(value: u8, mask: u8) -> bool {
    (value ^ mask) & mask == mask
}

pub fn build_string(bytes: &Vec<u8>) -> String {
    bytes.iter().map(|f| {*f as char}).collect()
}


#[cfg(test)]
mod byte_tests {
    use crate::bytes::{check_bits_set, check_bits_unset, combine_bytes, split_bytes};


    #[test]
    fn combine_bytes_test() {
        assert_eq!(0x1234_u16, combine_bytes(0x12, 0x34));
    }
    #[test]
    fn split_bytes_test() {
        let bytes = split_bytes(0x1234);
        assert_eq!(bytes.0, 0x34);
        assert_eq!(bytes.1, 0x12);
    }
    #[test]
    fn check_bits_set_test() {
        assert!(check_bits_set(0b1010_1010, 0b1010_0000));
        assert!(!check_bits_set(0b1111_0000, 0b0001_1000));
    }
    #[test]
    fn check_bits_unset_test() {
        assert!(check_bits_unset(0b1010_1010, 0b0101_0000));
        assert!(!check_bits_unset(0b0001_0000, 0b0001_1000));
    }

}
