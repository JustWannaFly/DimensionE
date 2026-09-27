// TODO acutally implement this
pub fn _build_status_flags(
    curr_flags: u8,
    flag: Option<bool>,
) -> u8 {
    let mut maintain_mask: u8 = 0x0;
    let mut set_mask: u8 = 0x0;
    match flag {
        Some(is_set) => {
            if is_set {
                set_mask += 0b1000_0000
            }
        }
        None => maintain_mask += 0b1000_0000,
    }
    (curr_flags & maintain_mask) | set_mask
}