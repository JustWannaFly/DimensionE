use crate::{
    bytes::{combine_bytes, split_bytes},
    snes::{
        cpu::{
            ProcessingState::{LoadingOp, Processing},
            instrusctions::{AddressMode, get_op_size},
        },
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

const DEFAULT_TICKS_PER_CYCLE: u8 = 6;

enum ProcessingState {
    LoadingOp,
    Processing,
}

struct ProcessingDetails {
    op: Operation,
    args: Vec<u8>,
    operand: Option<u16>,
    op_cycles: u8,
    write_in_progress: bool,
}
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
    ticks_per_cycle: u8,
    ticks_since_cycle: u8,

    processing_state: ProcessingState,
    processing_data: ProcessingDetails,
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
            ticks_per_cycle: DEFAULT_TICKS_PER_CYCLE,
            ticks_since_cycle: DEFAULT_TICKS_PER_CYCLE,

            processing_state: LoadingOp,
            processing_data: ProcessingDetails {
                op: Operation::NOP,
                args: vec![],
                operand: None,
                op_cycles: 0,
                write_in_progress: false,
            },
        }
    }
    pub fn initialize(&mut self, program_start: u16) {
        // TODO initialize all registers
        self.ticks_per_cycle = DEFAULT_TICKS_PER_CYCLE;
        self.ticks_since_cycle = DEFAULT_TICKS_PER_CYCLE;

        self.data_bank = 0x0000;
        self.direct_page = 0x0000_0000;
        self.program_bank = 0x0000_0000;
        self.stack_pointer = 0x01FF;
        self.emulation_flag = true;
        self.program_counter = program_start;
        self.processing_state = LoadingOp;
        self.processing_data = ProcessingDetails {
            op: Operation::NOP,
            args: vec![],
            operand: None,
            op_cycles: 0,
            write_in_progress: false,
        };
    }
    pub fn tick(&mut self, memory_controller: &mut MemoryController) {
        self.ticks_since_cycle += 1;
        if self.ticks_since_cycle >= self.ticks_per_cycle {
            println!("");
            self.processing_data.op_cycles += 1;
            self.ticks_since_cycle = 0;
            self.process_cycle(memory_controller);
        }
        print!("|");
    }
    fn process_cycle(&mut self, memory_controller: &mut MemoryController) {
        match &self.processing_state {
            LoadingOp => {
                println!("Cycles for op: [{}]", self.processing_data.op_cycles);
                self.set_bus_location(memory_controller, self.program_bank, self.program_counter);
                let op_code = memory_controller.cpu_bus_read();
                self.processing_data = ProcessingDetails {
                    op: lookup_opcode(op_code),
                    args: vec![],
                    operand: None,
                    op_cycles: 0,
                    write_in_progress: false,
                };
                println!(
                    "Next instruction: [{:X}:{:X}]:[{:X}:{:?}]",
                    self.program_bank, self.program_counter, op_code, self.processing_data.op
                );
                self.processing_state = Processing;
            }
            Processing => {
                self.execute_op_cycle(memory_controller);
            }
        }
    }

    fn execute_op_cycle(&mut self, memory_controller: &mut MemoryController) {
        // TODO add per cycle logging option
        match self.processing_data.op {
            Operation::SEI => {
                self.update_processor_status(None, None, Some(true), None, None, None, None, None);

                self.advance_program_counter(1);
            }
            Operation::STZ(address_mode) => match self.processing_data.operand {
                Some(address) => {
                    if !self.processing_data.write_in_progress {
                        self.set_bus_location(memory_controller, 0, address);
                        memory_controller.cpu_bus_write(0);
                        self.processing_data.write_in_progress = true
                    } else {
                        self.set_bus_location(memory_controller, 0, address.wrapping_add(1));
                        memory_controller.cpu_bus_write(0);
                        self.advance_program_counter_with_address_mode(&address_mode);
                    }
                }
                None => {
                    self.cycle_operand_loading(&address_mode, memory_controller, false);
                }
            },
            _ => todo!("[Operation::{:?}]", self.processing_data.op),
        }
    }

    fn cycle_operand_loading(
        &mut self,
        address_mode: &AddressMode,
        memory_controller: &mut MemoryController,
        is8bit: bool,
    ) {
        self.load_one_arg(memory_controller);
        let mut arg_count = get_op_size(address_mode) - 1;
        if is8bit && matches!(address_mode, AddressMode::Immediate) {
            arg_count -= 1;
        }
        if (self.processing_data.args.len() as u16) < (arg_count) {
            return ();
        }
        self.processing_data.operand = Some(self.build_operand_from_args(address_mode));
    }
    fn build_operand_from_args(&self, address_mode: &AddressMode) -> u16 {
        match address_mode {
            AddressMode::Absolute => combine_bytes(self.processing_data.args[1], self.processing_data.args[0]),

            _ => todo!("[AddressMode::{:?}]", address_mode),
        }
    }
    fn set_bus_location(
        &mut self,
        memory_controller: &mut MemoryController,
        bank: u8,
        address: u16,
    ) {
        self.ticks_per_cycle = memory_controller.cpu_bus_set_location(bank, address);
    }
    fn load_one_arg(&mut self, memory_controller: &mut MemoryController) {
        self.set_bus_location(
            memory_controller,
            self.program_bank,
            self.program_counter
                .wrapping_add(1)
                .wrapping_add(self.processing_data.args.len() as u16),
        );
        self.processing_data.args.push(memory_controller.cpu_bus_read());
    }
    fn _resolve_address(
        &self,
        memory_controller: &MemoryController,
        address_mode: &AddressMode,
    ) -> Option<(u8, u16)> {
        let bank;
        let address;
        let args = vec![];

        match address_mode {
            AddressMode::Absolute => {
                bank = self.data_bank;
                address = combine_bytes(args[1], args[0]);
            }
            AddressMode::AbsoluteIndirect => {
                bank = 0x00;
                let pointer = combine_bytes(args[1], args[0]);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteIndirectLong => {
                let pointer = combine_bytes(args[1], args[0]);
                let addr_bytes = memory_controller.read(0, pointer, 3);
                bank = addr_bytes[2];
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteIndirectX => {
                bank = 0x00;
                let pointer = self.x.wrapping_add(combine_bytes(args[1], args[0]));
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::AbsoluteLong => {
                bank = args[0];
                address = combine_bytes(args[2], args[1]);
            }
            AddressMode::AbsoluteLongX => {
                bank = args[2];
                address = self.x.wrapping_add(combine_bytes(args[1], args[0]));
            }
            AddressMode::AbsoluteX => {
                bank = self.data_bank;
                address = self.x.wrapping_add(combine_bytes(args[1], args[0]));
            }
            AddressMode::AbsoluteY => {
                bank = self.data_bank;
                address = self.y.wrapping_add(combine_bytes(args[1], args[0]));
            }
            AddressMode::Accumulator => {
                bank = self.data_bank; // TODO verify
                address = self.accumulator;
            }
            AddressMode::DirectPage => {
                bank = 0x00;
                address = self.direct_page.wrapping_add(args[0] as u16);
            }
            AddressMode::DirectPageIndirect => {
                bank = self.data_bank;
                let pointer = self.direct_page.wrapping_add(args[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectLong => {
                let pointer = self.direct_page.wrapping_add(args[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 3);
                bank = addr_bytes[2];
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectLongY => {
                let pointer = self.direct_page.wrapping_add(args[0] as u16);
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
                    .wrapping_add(self.x.wrapping_add(args[0] as u16));
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = combine_bytes(addr_bytes[1], addr_bytes[0]);
            }
            AddressMode::DirectPageIndirectY => {
                bank = self.data_bank;
                let pointer = self.direct_page.wrapping_add(args[0] as u16);
                let addr_bytes = memory_controller.read(0, pointer, 2);
                address = self
                    .y
                    .wrapping_add(combine_bytes(addr_bytes[1], addr_bytes[0]));
            }
            AddressMode::DirectPageX => {
                bank = 0x00;
                address = self
                    .direct_page
                    .wrapping_add(self.x.wrapping_add(args[0] as u16));
            }
            AddressMode::DirectPageY => {
                bank = 0x00;
                address = self
                    .direct_page
                    .wrapping_add(self.y.wrapping_add(args[0] as u16));
            }
            AddressMode::Immediate => return None,
            AddressMode::StackRelative => {
                bank = 0x00;
                address = self.stack_pointer.wrapping_add(args[0] as u16);
            }
            AddressMode::StackRelativeIndirectY => {
                bank = self.data_bank;
                let pointer = self.stack_pointer.wrapping_add(args[0] as u16);
                let addr_bytes = memory_controller.read(bank, pointer, 2);
                address = self
                    .y
                    .wrapping_add(combine_bytes(addr_bytes[1], addr_bytes[0]));
            }
        }

        Some((bank, address))
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
    fn advance_program_counter_with_address_mode(&mut self, address_mode: &AddressMode) {
        let is8bit = self.processor_status & ACCUMULATOR_MASK != 0;
        self.advance_program_counter_address_mode_and_8bit(address_mode, is8bit);
    }
    fn advance_program_counter_address_mode_and_8bit(
        &mut self,
        address_mode: &AddressMode,
        is8bit: bool,
    ) {
        let mut op_size = instrusctions::get_op_size(address_mode);
        if matches!(address_mode, AddressMode::Immediate) && is8bit {
            op_size -= 1;
        }
        self.advance_program_counter(op_size);
    }
    fn advance_program_counter(&mut self, op_size: u16) {
        self.processing_state = LoadingOp;
        self.program_counter = self.program_counter.wrapping_add(op_size);
    }
    fn stack_push_8(&mut self, memory_controller: &mut MemoryController, value: u8) {
        memory_controller.write(0x00, self.stack_pointer, value);
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
