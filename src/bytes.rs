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
    bytes.iter().map(|f| *f as char).collect()
}
/// reads a u16 as a 4 digit BDC encoded number
pub fn get_bdc_value(word: u16) -> u16 {
    let low_bit_mask = 0x0F_u16;
    let ones = word & low_bit_mask;
    let tens = (word >> 4) & low_bit_mask;
    let hundreds = (word >> 8) & low_bit_mask;
    let thousands = (word >> 12) & low_bit_mask;
    (thousands * 1000) + (hundreds * 100) + (tens * 10) + ones
}
/// takes a 4 digit number and encodes it in BDC encoding
pub fn encode_bdc(bdc_value: u16) -> u16 {
    let mut working_value = bdc_value;
    let ones = working_value % 10;
    working_value = working_value / 10;
    let tens = working_value % 10;
    working_value = working_value / 10;
    let hundreds = working_value % 10;
    working_value = working_value / 10;
    let thousands = working_value;
    (thousands << 12) | (hundreds << 8) | (tens << 4) | ones
}

#[cfg(test)]
mod byte_tests {
    use crate::bytes::{
        check_bits_set, check_bits_unset, combine_bytes, encode_bdc, get_bdc_value, split_bytes,
    };
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
    #[test]
    fn get_bdc_value_test() {
        assert_eq!(9753_u16, get_bdc_value(0b1001_0111_0101_0011));
    }
    #[test]
    fn encode_bdc_test() {
        assert_eq!(encode_bdc(9753), 0b1001_0111_0101_0011);
    }
}
