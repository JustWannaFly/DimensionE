use std::{mem, panic::Location};

use crate::{
    bytes::{combine_bytes, split_bytes},
    snes::{
        cpu::instrusctions::{AddressMode, OpDetail, Operation::JSL},
        memory::MemoryController,
    },
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
        self.stack_pointer = 0x01FF;
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
            Operation::ADC(detail) => {
                let mut sum: u16;
                let flag_shift: u8;
                let operand = self.fetch_operand(memory_controller, &detail);
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    flag_shift = 15;
                } else {
                    flag_shift = 7
                }
                if self.processor_status & DECIMAL_MASK == 0 {
                    sum = self.accumulator.wrapping_add(operand);
                    // 8 bit wrapping support
                    if (self.processor_status & ACCUMULATOR_MASK == 1) && sum > 0xFF {
                        sum -= 0xFF;
                    }
                } else {
                    todo!("binary coded addition not yet implemented")
                }
                let is_carry = sum >> flag_shift == 1
                    && operand >> flag_shift == 0
                    && self.accumulator >> flag_shift == 0;
                let is_overflow = (self.accumulator >> flag_shift == operand >> flag_shift)
                    && (self.accumulator >> flag_shift != sum >> flag_shift);
                let is_zero = sum == 0;
                let is_negative = sum >> flag_shift != 0;

                self.update_processor_status(
                    Some(is_carry),
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    Some(is_overflow),
                    Some(is_negative),
                );
                self.advance_program_counter_with_detail(&detail);
            }
            Operation::AND(detail) => {
                let operand = self.fetch_operand(memory_controller, &detail);

                let result = self.accumulator & operand;
                let is_negative = if self.processor_status & ACCUMULATOR_MASK == 0 {
                    result >> 15 != 0
                } else {
                    result >> 7 != 0
                };
                let is_zero = result == 0;
                self.update_processor_status(
                    None,
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(is_negative),
                );
                self.advance_program_counter_with_detail(&detail);
            }
            Operation::CLC => {
                self.update_processor_status(Some(false), None, None, None, None, None, None, None);
                self.advance_program_counter(1);
            }
            Operation::CLD => {
                self.update_processor_status(None, None, None, Some(false), None, None, None, None);
                self.advance_program_counter(1);
            }
            Operation::CLI => {
                self.update_processor_status(None, None, Some(false), None, None, None, None, None);
                self.advance_program_counter(1);
            }
            Operation::CLV => {
                self.update_processor_status(None, None, None, None, None, None, Some(false), None);
                self.advance_program_counter(1);
            }
            Operation::JML(detail) => {
                let location = self.resolve_address(memory_controller, &detail).unwrap();
                self.program_bank = location.0;
                self.program_counter = location.1;
            }
            Operation::JMP(detail) => {
                let location = self.resolve_address(memory_controller, &detail).unwrap();
                self.program_counter = location.1;
            }
            Operation::JSL => {
                self.stack_push_8(memory_controller, self.program_bank);
                self.stack_push_16(memory_controller, self.program_counter.wrapping_add(4));
                let arg_bytes = self.read_op_args(memory_controller, 3);
                self.program_bank = arg_bytes[2];
                self.program_counter = combine_bytes(arg_bytes[1], arg_bytes[0]);
            }
            Operation::JSR(detail) => {
                self.stack_push_16(
                    memory_controller,
                    self.program_counter.wrapping_add(detail.op_size),
                );
                let location = self.resolve_address(memory_controller, &detail).unwrap();
                self.program_counter = location.1;
            }
            Operation::LDA(detail) => {
                let value = self.fetch_operand(memory_controller, &detail);
                let is_negative;
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    is_negative = value >> 15 != 0
                } else {
                    is_negative = value >> 7 != 0
                }
                let is_zero = value == 0;
                self.update_processor_status(
                    None,
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(is_negative),
                );
                self.accumulator = value;

                self.advance_program_counter_with_detail(&detail);
            }
            Operation::LDY(detail) => {
                let value;
                let is8bit = self.processor_status & INDEX_REG_MASK != 0;
                if is8bit {
                    value = combine_bytes(0x0000, self.fetch_operand8(memory_controller, &detail))
                } else {
                    value = self.fetch_operand16(memory_controller, &detail);
                }
                self.y = value;
                self.advance_program_counter_detail_and_8bit(&detail, is8bit);
            }
            Operation::NOP => {
                self.advance_program_counter(1);
            }
            Operation::ORA(detail) => {
                let flag_shift: u8;
                let operand = self.fetch_operand(memory_controller, &detail);
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    flag_shift = 15;
                } else {
                    flag_shift = 7
                }
                self.accumulator = self.accumulator | operand;
                let is_zero = self.accumulator == 0;
                let is_negative = self.accumulator >> flag_shift != 0;

                self.update_processor_status(
                    None,
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(is_negative),
                );

                self.advance_program_counter_with_detail(&detail);
            }
            Operation::PHA => {
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    self.stack_push_16(memory_controller, self.accumulator);
                } else {
                    self.stack_push_8(memory_controller, split_bytes(self.accumulator).0);
                }
                self.advance_program_counter(1);
            }
            Operation::PHB => {
                self.stack_push_8(memory_controller, self.data_bank);
                self.advance_program_counter(1);
            }
            Operation::PHD => {
                self.stack_push_16(memory_controller, self.direct_page);
                self.advance_program_counter(1);
            }
            Operation::PHK => {
                self.stack_push_8(memory_controller, self.program_bank);
                self.advance_program_counter(1);
            }
            Operation::PHP => {
                self.stack_push_8(memory_controller, self.processor_status);
                self.advance_program_counter(1);
            }
            Operation::PHX => {
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    self.stack_push_16(memory_controller, self.x);
                } else {
                    self.stack_push_8(memory_controller, split_bytes(self.x).0);
                }
                self.advance_program_counter(1);
            }
            Operation::PHY => {
                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    self.stack_push_16(memory_controller, self.y);
                } else {
                    self.stack_push_8(memory_controller, split_bytes(self.y).0);
                }
                self.advance_program_counter(1);
            }
            Operation::REP => {
                let clear_flags = self.read_op_args(memory_controller, 1)[0];
                let carry = if clear_flags & CARRY_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                let zero = if clear_flags & ZERO_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                let irq_disable = if clear_flags & IRQ_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                let decimal = if clear_flags & DECIMAL_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                let index_register = if clear_flags & INDEX_REG_MASK != 0 || self.emulation_flag {
                    Some(false)
                } else {
                    None
                };
                let accumulator = if clear_flags & ACCUMULATOR_MASK != 0 || self.emulation_flag {
                    Some(false)
                } else {
                    None
                };
                let overflow = if clear_flags & OVERFLOW_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                let negative = if clear_flags & NEGATIVE_MASK != 0 {
                    Some(false)
                } else {
                    None
                };
                self.update_processor_status(
                    carry,
                    zero,
                    irq_disable,
                    decimal,
                    index_register,
                    accumulator,
                    overflow,
                    negative,
                );
                self.advance_program_counter(2);
            }
            Operation::SEI => {
                self.update_processor_status(None, None, Some(true), None, None, None, None, None);
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            Operation::SEP => {
                let clear_flags = self.read_op_args(memory_controller, 1)[0];
                let carry = if clear_flags & CARRY_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                let zero = if clear_flags & ZERO_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                let irq_disable = if clear_flags & IRQ_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                let decimal = if clear_flags & DECIMAL_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                let index_register = if clear_flags & INDEX_REG_MASK != 0 || self.emulation_flag {
                    Some(true)
                } else {
                    None
                };
                let accumulator = if clear_flags & ACCUMULATOR_MASK != 0 || self.emulation_flag {
                    Some(true)
                } else {
                    None
                };
                let overflow = if clear_flags & OVERFLOW_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                let negative = if clear_flags & NEGATIVE_MASK != 0 {
                    Some(true)
                } else {
                    None
                };
                self.update_processor_status(
                    carry,
                    zero,
                    irq_disable,
                    decimal,
                    index_register,
                    accumulator,
                    overflow,
                    negative,
                );
                self.advance_program_counter(2);
            }
            Operation::STA(detail) => {
                let save_location = self.resolve_address(memory_controller, &detail).unwrap();
                let accumulator_bytes = split_bytes(self.accumulator);
                let write_bytes;

                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    write_bytes = vec![accumulator_bytes.0, accumulator_bytes.1]
                } else {
                    write_bytes = vec![accumulator_bytes.0]
                }

                memory_controller.write(save_location.0, save_location.1, write_bytes);
                self.advance_program_counter_with_detail(&detail);
            }
            Operation::STZ(detail) => {
                let save_location = self.resolve_address(memory_controller, &detail).unwrap();
                let write_bytes: Vec<u8>;

                if self.processor_status & ACCUMULATOR_MASK == 0 {
                    write_bytes = vec![0, 0]
                } else {
                    write_bytes = vec![0]
                }

                memory_controller.write(save_location.0, save_location.1, write_bytes);
                self.advance_program_counter_with_detail(&detail);
            }
            Operation::TCD => {
                self.direct_page = self.accumulator;
                self.advance_program_counter(1);
            }
            Operation::TCS => {
                self.stack_pointer = self.accumulator;
                self.advance_program_counter(1);
            }
            Operation::XCE => {
                let old_emulation_flag = self.emulation_flag;
                self.emulation_flag = (self.processor_status & 0b0000_0001) != 0;

                // when entering emulation mode index and accumulator are forced into 8 bit mode
                let accumulator;
                let index_register;
                if old_emulation_flag {
                    accumulator = Some(true);
                    index_register = Some(true);
                } else {
                    accumulator = None;
                    index_register = None;
                }
                self.update_processor_status(
                    Some(old_emulation_flag),
                    None,
                    None,
                    None,
                    index_register,
                    accumulator,
                    None,
                    None,
                );
                self.advance_program_counter(1);
            }
            _ => todo!(
                "Operation function not implemented for op_code: [{:X}]",
                op_code
            ),
        }
    }
    fn resolve_address(
        &self,
        memory_controller: &MemoryController,
        op_detail: &OpDetail,
    ) -> Option<(u8, u16)> {
        let bank;
        let address;
        let bytes = self.read_op_args(memory_controller, op_detail.op_size - 1);

        match op_detail.address_mode {
            AddressMode::Absolute => {
                bank = self.data_bank;
                address = combine_bytes(bytes[1], bytes[0]);
            }
            AddressMode::AbsoluteIndirect => {
                bank = 0x00;
                let pointer = combine_bytes(bytes[1], bytes[0]);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteIndirectLong => {
                let pointer = combine_bytes(bytes[1], bytes[0]);
                let addr_bytes = memory_controller.read(0, pointer, 3);
                bank = addr_bytes[2];
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteIndirectX => {
                bank = 0x00;
                let pointer = self.x.wrapping_add(combine_bytes(bytes[1], bytes[0]));
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteLong => {
                bank = bytes[0];
                address = combine_bytes(bytes[2], bytes[1]);
            }
            AddressMode::AbsoluteLongX => {
                bank = bytes[2];
                address = self.x.wrapping_add(combine_bytes(bytes[1], bytes[0]));
            }
            AddressMode::AbsoluteX => {
                bank = self.data_bank;
                address = self.x.wrapping_add(combine_bytes(bytes[1], bytes[0]));
            }
            AddressMode::AbsoluteY => {
                bank = self.data_bank;
                address = self.y.wrapping_add(combine_bytes(bytes[1], bytes[0]));
            }
            AddressMode::Accumulator => {
                bank = self.data_bank; // TODO verify
                address = self.accumulator;
            }
            AddressMode::DirectPage => {
                bank = 0x00;
                address = self.direct_page.wrapping_add(bytes[0] as u16);
            }
            AddressMode::DirectPageIndirect => {
                bank = self.data_bank;
                let pointer = self.direct_page.wrapping_add(bytes[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectLong => {
                let pointer = self.direct_page.wrapping_add(bytes[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 3);
                bank = addr_bytes[2];
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectLongY => {
                let pointer = self.direct_page.wrapping_add(bytes[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 3);
                bank = addr_bytes[2];
                address = self
                    .y
                    .wrapping_add(combine_bytes(addr_bytes[1], addr_bytes[0]));
            }
            AddressMode::DirectPageIndirectX => {
                bank = self.data_bank;
                let pointer = self
                    .direct_page
                    .wrapping_add(self.x.wrapping_add(bytes[0] as u16));
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectY => {
                bank = self.data_bank;
                let pointer = self.direct_page.wrapping_add(bytes[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = self
                    .y
                    .wrapping_add(combine_bytes(addr_bytes[1], addr_bytes[0]));
            }
            AddressMode::DirectPageX => {
                bank = 0x00;
                address = self
                    .direct_page
                    .wrapping_add(self.x.wrapping_add(bytes[0] as u16));
            }
            AddressMode::DirectPageY => {
                bank = 0x00;
                address = self
                    .direct_page
                    .wrapping_add(self.y.wrapping_add(bytes[0] as u16));
            }
            AddressMode::Immediate => return None,
            AddressMode::StackRelative => {
                bank = 0x00;
                address = self.stack_pointer.wrapping_add(bytes[0] as u16);
            }
            AddressMode::StackRelativeIndirectY => {
                bank = self.data_bank;
                let pointer = self.stack_pointer.wrapping_add(bytes[0] as u16);
                let addr_bytes = memory_controller.read(bank, pointer, 2);
                address = self
                    .y
                    .wrapping_add(combine_bytes(addr_bytes[1], addr_bytes[0]));
            }
        }

        Some((bank, address))
    }
    fn read_op_args(&self, memory_controller: &MemoryController, length: u16) -> Vec<u8> {
        memory_controller.read(
            self.program_bank,
            self.program_counter.wrapping_add(1),
            length,
        )
    }
    fn fetch_operand(&self, memory_controller: &MemoryController, detail: &OpDetail) -> u16 {
        if self.processor_status & ACCUMULATOR_MASK == 0 {
            // 16bit mode
            return self.fetch_operand16(memory_controller, detail);
        } else {
            // 8 bit
            return combine_bytes(0x0000, self.fetch_operand8(memory_controller, detail));
        }
    }
    fn fetch_operand8(&self, memory_controller: &MemoryController, detail: &OpDetail) -> u8 {
        let lookup_location = self.resolve_address(memory_controller, detail);
        match lookup_location {
            Some(location) => {
                return memory_controller.read_byte(location.0, location.1);
            }
            None => {
                return self.read_op_args(memory_controller, 1)[0];
            }
        }
    }
    fn fetch_operand16(&self, memory_controller: &MemoryController, detail: &OpDetail) -> u16 {
        let lookup_location = self.resolve_address(memory_controller, detail);
        match lookup_location {
            Some(location) => {
                let operand_bytes = memory_controller.read(location.0, location.1, 2);
                return combine_bytes(operand_bytes[1], operand_bytes[0]);
            }
            None => {
                let bytes = self.read_op_args(memory_controller, 2);
                return combine_bytes(bytes[1], bytes[0]);
            }
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
    /// When setting index registers to 8 bit this will also clear the high order bits of X and Y
    fn update_processor_status(
        &mut self,
        carry: Option<bool>,
        zero: Option<bool>,
        irq_disable: Option<bool>,
        decimal_mode: Option<bool>,
        index_register_select: Option<bool>,
        accumulator_select: Option<bool>,
        overflow: Option<bool>,
        negative: Option<bool>,
    ) {
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
                if is_set || self.emulation_flag {
                    set_mask += INDEX_REG_MASK;
                    self.x = self.x & 0x00FF;
                    self.y = self.y & 0x00FF;
                }
            }
            None => maintain_mask += INDEX_REG_MASK,
        }
        match accumulator_select {
            Some(is_set) => {
                if is_set || self.emulation_flag {
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
        self.processor_status = (self.processor_status & maintain_mask) | set_mask
    }
    fn advance_program_counter_with_detail(&mut self, op_detail: &OpDetail) {
        let is8bit = self.processor_status & ACCUMULATOR_MASK != 0;
        self.advance_program_counter_detail_and_8bit(op_detail, is8bit);
    }
    fn advance_program_counter_detail_and_8bit(&mut self, op_detail: &OpDetail, is8bit: bool) {
        let mut op_size = op_detail.op_size;
        if matches!(op_detail.address_mode, AddressMode::Immediate)
            && is8bit
        {
            op_size -= 1;
        }
        self.advance_program_counter(op_size);
    }
    fn advance_program_counter(&mut self, op_size: u16) {
        self.program_counter = self.program_counter.wrapping_add(op_size);
    }
    fn stack_push_8(&mut self, memory_controller: &mut MemoryController, value: u8) {
        memory_controller.write(0x00, self.stack_pointer, vec![value]);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }
    fn stack_push_16(&mut self, memory_controller: &mut MemoryController, value: u16) {
        let bytes = split_bytes(value);
        self.stack_push_8(memory_controller, bytes.1);
        self.stack_push_8(memory_controller, bytes.0);
    }
    fn stack_pop_8(&mut self, memory_controller: &MemoryController) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        memory_controller.read_byte(0x00, self.stack_pointer)
    }
    fn stack_pop_16(&mut self, memory_controller: &MemoryController) -> u16 {
        let low_byte = self.stack_pop_8(memory_controller);
        let high_byte = self.stack_pop_8(memory_controller);
        combine_bytes(high_byte, low_byte)
    }
}
