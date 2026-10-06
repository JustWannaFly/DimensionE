use crate::{
    bytes::{combine_bytes, split_bytes},
    snes::{cpu::instrusctions::AddressMode, memory::MemoryController},
};

mod instrusctions;
use instrusctions::{Operation, lookup_opcode};

const CARRY_MASK: u8 = 0b0000_0001;
const ZERO_MASK: u8 = 0b0000_0010;
const IRQ_MASK: u8 = 0b0000_0100;
const DECIMAL_MASK: u8 = 0b0000_1000;
const INDEX_REG_MASK: u8 = 0b0001_0000;
const ACCUMULATOR_MASK: u8 = 0b0010_0000;
const OVERFLOW_MASK: u8 = 0b0100_0000;
const NEGATIVE_MASK: u8 = 0b1000_0000;

pub struct CPU {
    // TODO add support for emulation mode & decimal mode
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
            Operation::ADC(address_mode) => {
                let mut sum: u16;
                let flag_shift: u8;
                let operand: u16;
                let lookup_location = self.resolve_address(memory_controller, &address_mode);
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    // 16 bit
                    let operand_bytes =
                        memory_controller.read(lookup_location.0, lookup_location.1, 2);
                    operand = combine_bytes(operand_bytes[1], operand_bytes[0]);
                    flag_shift = 15;
                } else {
                    // 8 bit
                    operand = combine_bytes(
                        0x0000,
                        memory_controller.read_byte(lookup_location.0, lookup_location.1),
                    );
                    flag_shift = 7
                }
                if self.processor_status & DECIMAL_MASK == 0 {
                    sum = self.accumulator.wrapping_add(operand);
                    // 8 bit wrapping support
                    if (self.processor_status & ACCUMULATOR_MASK == 1) && sum > 0xFF {
                        sum -= 0xFF;
                    }
                } else {
                    panic!("binary coded addition not yet implemented")
                }
                let is_carry = sum >> flag_shift == 1
                    && operand >> flag_shift == 0
                    && self.accumulator >> flag_shift == 0;
                let is_overflow = (self.accumulator >> flag_shift == operand >> flag_shift)
                    && (self.accumulator >> flag_shift != sum >> flag_shift);
                let is_zero = sum == 0;
                let is_negative = sum >> flag_shift != 0;

                self.processor_status = build_status_flags(
                    self.processor_status,
                    Some(is_carry),
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    Some(is_overflow),
                    Some(is_negative),
                );
                self.program_counter = self
                    .program_counter
                    .wrapping_add(1 + get_arg_byte_count(&address_mode))
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
            Operation::JMP(address_mode) => {
                let arg_bytes =
                    self.read_op_args(memory_controller, get_arg_byte_count(&address_mode));
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
            Operation::STA(address_mode) => {
                let save_location = self.resolve_address(memory_controller, &address_mode);
                let accumulator_bytes = split_bytes(self.accumulator);
                let write_bytes;

                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    write_bytes = vec![accumulator_bytes.0, accumulator_bytes.1]
                } else {
                    write_bytes = vec![accumulator_bytes.0]
                }

                memory_controller.write(save_location.0, save_location.1, write_bytes);
                self.program_counter = self
                    .program_counter
                    .wrapping_add(1 + get_arg_byte_count(&address_mode));
            }
            Operation::STZ(address_mode) => {
                let save_location = self.resolve_address(memory_controller, &address_mode);
                let write_bytes: Vec<u8>;

                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    write_bytes = vec![0,0]
                } else {
                    write_bytes = vec![0]
                }

                memory_controller.write(save_location.0, save_location.1, write_bytes);
                self.program_counter = self
                    .program_counter
                    .wrapping_add(1 + get_arg_byte_count(&address_mode));
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
    fn resolve_address(
        &self,
        memory_controller: &MemoryController,
        address_mode: &AddressMode,
    ) -> (u8, u16) {
        let bank;
        let address;
        let bytes = self.read_op_args(memory_controller, get_arg_byte_count(address_mode));

        match address_mode {
            AddressMode::Absolute => {
                bank = self.data_bank;
                address = combine_bytes(bytes[1], bytes[0]);
            }
            AddressMode::AbsoluteIndexX => {
                bank = self.data_bank;
                address = self.x + combine_bytes(bytes[1], bytes[0]);
            }
            AddressMode::AbsoluteIndexY => {
                bank = self.data_bank;
                address = self.y + combine_bytes(bytes[1], bytes[0]);
            }
            AddressMode::AbsoluteLong => {
                bank = bytes[0];
                address = combine_bytes(bytes[2], bytes[1]);
            }
            AddressMode::AbsoluteLongIndexX => {
                bank = todo!("need to add support for AbsoluteLongIndex");
                address = todo!("need to add support for AbsoluteLongIndex");
            }
            AddressMode::DirectPage => {
                bank = 0x00;
                address = self.direct_page.wrapping_add(bytes[0] as u16);
            }
            AddressMode::DirectPageIndexX => {
                bank = todo!("need to add support for DirectPageIndexX");
                address = todo!("need to add support for DirectPageIndexX");
            }
        }

        (bank, address)
    }
    fn read_op_args(&self, memory_controller: &MemoryController, length: u16) -> Vec<u8> {
        memory_controller.read(
            self.program_bank,
            self.program_counter.wrapping_add(1),
            length,
        )
    }
}

fn get_arg_byte_count(address_mode: &AddressMode) -> u16 {
    match address_mode {
        AddressMode::Absolute => 2,
        AddressMode::AbsoluteIndexX => 2,
        AddressMode::AbsoluteIndexY => 2,
        AddressMode::AbsoluteLong => 3,
        AddressMode::AbsoluteLongIndexX => 3,
        AddressMode::DirectPage => 1,
        AddressMode::DirectPageIndexX => 1,
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
    let mut maintain_mask: u8 = 0x0;
    let mut set_mask: u8 = 0x0;
    match carry {
        Some(is_set) => {
            if is_set {
                set_mask += CARRY_MASK
            }
        }
        None => maintain_mask += CARRY_MASK,
    }
    match zero {
        Some(is_set) => {
            if is_set {
                set_mask += ZERO_MASK
            }
        }
        None => maintain_mask += ZERO_MASK,
    }
    match irq_disable {
        Some(is_set) => {
            if is_set {
                set_mask += IRQ_MASK
            }
        }
        None => maintain_mask += IRQ_MASK,
    }
    match decimal_mode {
        Some(is_set) => {
            if is_set {
                set_mask += DECIMAL_MASK
            }
        }
        None => maintain_mask += DECIMAL_MASK,
    }
    match index_register_select {
        Some(is_set) => {
            if is_set {
                set_mask += INDEX_REG_MASK
            }
        }
        None => maintain_mask += INDEX_REG_MASK,
    }
    match accumulator_select {
        Some(is_set) => {
            if is_set {
                set_mask += ACCUMULATOR_MASK
            }
        }
        None => maintain_mask += ACCUMULATOR_MASK,
    }
    match overflow {
        Some(is_set) => {
            if is_set {
                set_mask += OVERFLOW_MASK
            }
        }
        None => maintain_mask += OVERFLOW_MASK,
    }
    match negative {
        Some(is_set) => {
            if is_set {
                set_mask += NEGATIVE_MASK
            }
        }
        None => maintain_mask += NEGATIVE_MASK,
    }
    (curr_flags & maintain_mask) | set_mask
}
