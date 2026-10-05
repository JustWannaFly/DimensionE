use crate::{
    bytes::{combine_bytes, split_bytes}, snes::{cpu::instrusctions::AddressMode, memory::MemoryController},
};

mod instrusctions;
use instrusctions::{Operation, lookup_opcode};

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
    emulation_flag: bool,
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
            emulation_flag: true,
        }
    }
    pub fn initialize(&mut self, program_start: u16) {
        // TODO initialize all registers
        
        self.data_bank = 0x0000;
        self.direct_page = 0x0000_0000;
        self.program_bank = 0x0000_0000;
        self.emulation_flag = true;
        self.program_counter = program_start;
    }
    /// This will probably need to be broken out when implementing timing
    pub fn execute_instruction(&mut self, memory_controller: &mut MemoryController) {
        let op_code = memory_controller.read_byte(self.program_bank, self.program_counter);
        println!(
            "Processing [{:X}] at address [{:X}:{:X}]",
            op_code, self.program_bank, self.program_counter
        );
        let operation = lookup_opcode(op_code);
        match operation {
            Operation::ADC(details) => {
                // let value;
                let arg_bytes = read_op_args(
                    memory_controller,
                    self.program_bank,
                    self.program_counter,
                    details.arg_length,
                );
                let lookup_location = self.resolve_address(arg_bytes, details.address_mode);
                panic!("not done with adding")
            }
            Operation::CLC => {
                self.processor_status = build_status_flags(
                    self.processor_status,
                    Some(false),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                self.program_counter += 1;
            }
            Operation::CLD => {
                self.processor_status = build_status_flags(
                    self.processor_status,
                    None,
                    None,
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                    None,
                );
                self.program_counter += 1;
            }
            Operation::CLI => {
                self.processor_status = build_status_flags(
                    self.processor_status,
                    None,
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                self.program_counter += 1;
            }
            Operation::CLV => {
                self.processor_status = build_status_flags(
                    self.processor_status,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(false),
                    None,
                );
                self.program_counter += 1;
            }
            Operation::JMP(details) => {
                let arg_bytes = read_op_args(
                    memory_controller,
                    self.program_bank,
                    self.program_counter,
                    details.arg_length,
                );
                let dest_bank;
                let dest_address;
                match op_code {
                    0x5C => {
                        dest_bank = arg_bytes[0];
                        dest_address = combine_bytes(arg_bytes[2], arg_bytes[1]);
                    }
                    _ => panic!("Unsupported JMP op_code: [{:X}]", op_code),
                }
                self.program_bank = dest_bank;
                self.program_counter = dest_address;
            }
            Operation::NOP => {
                self.program_counter += 1;
            }
            Operation::SEI => {
                self.processor_status = build_status_flags(
                    self.processor_status,
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                self.program_counter += 1;
            }
            Operation::XCE => {
                let exchanged_flags = build_status_flags(
                    self.processor_status,
                    Some(self.emulation_flag),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                self.emulation_flag = (self.processor_status & 0b0000_0001) != 0;
                self.processor_status = exchanged_flags;
                self.program_counter += 1;
            }
            _ => panic!(
                "Operation function not implemented for op_code: [{:X}]",
                op_code
            ),
        }
    }
    pub fn resolve_address(&self, bytes: Vec<u8>, address_mode: AddressMode) -> (u8, u16) {
        let bank;
        let address;

        match address_mode {
            AddressMode::AbsoluteLong => {
                bank = bytes[0];
                address = combine_bytes(bytes[2], bytes[1]);
            }
            AddressMode::DirectPage => {
                bank = 0x00;
                address = self.direct_page + bytes[0] as u16;
            }
        }

        (bank, address)
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
fn build_status_flags(
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
    let carry_mask = 0b0000_0001;
    let zero_mask = 0b0000_0010;
    let irq_mask = 0b0000_0100;
    let decimal_mask = 0b0000_1000;
    let index_reg_mask = 0b0001_0000;
    let accumulator_mask = 0b0010_0000;
    let overflow_mask = 0b0100_0000;
    let negative_mask = 0b1000_0000;

    let mut maintain_mask: u8 = 0x0;
    let mut set_mask: u8 = 0x0;
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

fn read_op_args(
    memory_controller: &MemoryController,
    bank: u8,
    op_address: u16,
    length: u8,
) -> Vec<u8> {
    memory_controller.read(bank, op_address.wrapping_add(1), length)
}
