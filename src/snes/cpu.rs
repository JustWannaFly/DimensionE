use crate::snes::memory::MemoryController;

mod instrusctions;
use instrusctions::{
    Operation,
    lookup_opcode,
};

pub struct CPU {
    accumulator: u16,
    x: u16,
    y: u16,
    direct_page: u16,
    stack_pointer: u16,
    processor_status: u8,
    data_bank: u8,
    program_bank: u8,
    program_counter: u16,

}
impl CPU {
    pub fn new() -> CPU {
        CPU {
            accumulator: 0x0000_0000,
            x: 0x0000_0000,
            y: 0x0000_0000,
            direct_page: 0x0000_0000,
            stack_pointer: 0x0000_0000,
            processor_status: 0x0000,
            data_bank: 0x0000,
            program_bank: 0x0000,
            program_counter: 0x0000_0000,
        }
    }
    pub fn initialize(&mut self, program_start: u16) {
        self.program_counter = program_start
    }
    /// This will probably need to be broken out when implementing timing
    pub fn execute_instruction(&mut self, memory_controller: &mut MemoryController) {
        let operation = lookup_opcode(memory_controller.read_byte(self.data_bank, self.program_counter));
    }
}

/// Processor flags
/// Bits: 7   6   5   4   3   2   1   0
///                                  |e├─── Emulation: 0 = Native Mode
///      |n| |v| |m| |x| |d| |i| |z| |c|
///      └┼───┼───┼───┼───┼───┼───┼───┼┘
///       │   │   │   │   │   │   │   └──────── Carry: 1 = Carry set
///       │   │   │   │   │   │   └───────────── Zero: 1 = Result is zero
///       │   │   │   │   │   └────────── IRQ Disable: 1 = Disabled
///       │   │   │   │   └───────────── Decimal Mode: 1 = Decimal, 0 = Hexadecimal
///       │   │   │   └──────── Index Register Select: 1 = 8-bit, 0 = 16-bit
///       │   │   └─────────────── Accumulator Select: 1 = 8-bit, 0 = 16-bit
///       │   └───────────────────────────── Overflow: 1 = Overflow set
///       └───────────────────────────────── Negative: 1 = Negative set
///
/// *note: when setting the accumulator into 8-bit mode, the caller is responsible
///     for clearing the high bits
pub fn _build_status_flags(
    curr_flags: u8,
    carry: Option<bool>,
    zero: Option<bool>,
    irq_disable: Option<bool>,
    decimal_mode: Option<bool>,
    index_register_select: Option<bool>,
    accumulator_select: Option<bool>,
    overflow: Option<bool>,
    negative: Option<bool>,
) -> u8 {
    let mut maintain_mask: u8 = 0x0;
    let mut set_mask: u8 = 0x0;
    let carry_mask = 0b0000_0001;
    let zero_mask = 0b0000_0010;
    let irq_mask = 0b0000_0100;
    let decimal_mask = 0b0000_1000;
    let index_reg_mask = 0b0001_0000;
    let accumulator_mask = 0b0010_0000;
    let overflow_mask = 0b0100_0000;
    let negative_mask = 0b1000_0000;
    match carry {
        Some(is_set) => {
            if is_set {
                set_mask += carry_mask
            }
        }
        None => maintain_mask += carry_mask,
    }
    match zero {
        Some(is_set) => {
            if is_set {
                set_mask += zero_mask
            }
        }
        None => maintain_mask += zero_mask,
    }
    match irq_disable {
        Some(is_set) => {
            if is_set {
                set_mask += irq_mask
            }
        }
        None => maintain_mask += irq_mask,
    }
    match decimal_mode {
        Some(is_set) => {
            if is_set {
                set_mask += decimal_mask
            }
        }
        None => maintain_mask += decimal_mask,
    }
    match index_register_select {
        Some(is_set) => {
            if is_set {
                set_mask += index_reg_mask
            }
        }
        None => maintain_mask += index_reg_mask,
    }
    match accumulator_select {
        Some(is_set) => {
            if is_set {
                set_mask += accumulator_mask
            }
        }
        None => maintain_mask += accumulator_mask,
    }
    match overflow {
        Some(is_set) => {
            if is_set {
                set_mask += overflow_mask
            }
        }
        None => maintain_mask += overflow_mask,
    }
    match negative {
        Some(is_set) => {
            if is_set {
                set_mask += negative_mask
            }
        }
        None => maintain_mask += negative_mask,
    }
    (curr_flags & maintain_mask) | set_mask
}
