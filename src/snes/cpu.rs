use crate::{
    bytes::{check_bits_set, combine_bytes, split_bytes},
    snes::{
        cpu::{
            ProcessingState::{LoadingOp, Processing},
            op_impls::xce,
            ops::{AddressMode, get_op_size},
        },
        memory::MemoryController,
    },
};

mod ops;
use ops::{Operation, lookup_opcode};
mod op_impls;

const CARRY_MASK: u8 = 0b0000_0001;
const ZERO_MASK: u8 = 0b0000_0010;
const IRQ_MASK: u8 = 0b0000_0100;
const DECIMAL_MASK: u8 = 0b0000_1000;
const INDEX_REG_MASK: u8 = 0b0001_0000;
const ACCUMULATOR_MASK: u8 = 0b0010_0000;
const OVERFLOW_MASK: u8 = 0b0100_0000;
const NEGATIVE_MASK: u8 = 0b1000_0000;

const BASE_CPU_SPEED: u8 = 6;

#[derive(Debug)]
pub enum ProcessingState {
    LoadingOp,
    Processing,
}

pub struct ProcessingDetails {
    pub op: Operation,
    pub args: Vec<u8>,
    pub operand: Option<u16>,
    pub op_cycles: u8,
    pub write_count: u8,
    pub read_count: u8,
    pub op_error: bool,
    pub address_error: bool,
    pub debug_pause: bool,
}
pub struct CPU {
    // TODO add support for emulation mode & decimal mode
    pub accumulator: u16,
    pub x: u16,
    pub y: u16,
    pub direct_page: u16,
    pub stack_pointer: u16,
    pub processor_status: u8,
    pub data_bank: u8,
    pub program_bank: u8,
    pub program_counter: u16,
    pub emulation_flag: bool,
    memory_access_delay: Option<u8>,
    ticks_since_cycle: u8,

    pub processing_state: ProcessingState,
    pub processing_data: ProcessingDetails,
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
            memory_access_delay: None,
            ticks_since_cycle: BASE_CPU_SPEED,

            processing_state: LoadingOp,
            processing_data: ProcessingDetails {
                op: Operation::NOP,
                args: vec![],
                operand: None,
                op_cycles: 0,
                write_count: 0,
                read_count: 0,
                op_error: false,
                address_error: false,
                debug_pause: false,
            },
        }
    }
    pub fn initialize(&mut self, program_start: u16) {
        // TODO initialize all registers
        self.memory_access_delay = None;
        self.ticks_since_cycle = BASE_CPU_SPEED;

        self.processor_status = 0x00;
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
            write_count: 0,
            read_count: 0,
            op_error: false,
            address_error: false,
            debug_pause: false,
        };
    }
    // TODO refactor cpu to hold a reference to the memory controller so it doesn't need to be passed everywhere
    pub fn tick(&mut self, memory_controller: &mut MemoryController) {
        if !(self.processing_data.op_error || self.processing_data.address_error) {
            self.ticks_since_cycle += 1;
            let ticks_to_delay = self.memory_access_delay.unwrap_or(BASE_CPU_SPEED);
            if self.ticks_since_cycle >= ticks_to_delay {
                match self.memory_access_delay {
                    Some(_) => {
                        self.memory_access_delay = None;
                    }
                    None => (),
                }
                // println!("");
                self.ticks_since_cycle = 0;
                self.process_cycle(memory_controller);
            }
            // print!("|");
        }
    }
    pub fn process_cycle(&mut self, memory_controller: &mut MemoryController) {
        match &self.processing_state {
            LoadingOp => {
                println!("Cycles taken for op: [{}]", self.processing_data.op_cycles);
                let op_code =
                    self.read_byte(memory_controller, self.program_bank, self.program_counter);
                self.processing_data = ProcessingDetails {
                    op: lookup_opcode(op_code),
                    args: vec![],
                    operand: None,
                    op_cycles: 1,
                    write_count: 0,
                    read_count: 0,
                    op_error: false,
                    address_error: false,
                    debug_pause: false,
                };
                println!(
                    "Next instruction: [{:X}:{:X}]:[{:X}:{:?}]",
                    self.program_bank, self.program_counter, op_code, self.processing_data.op
                );
                self.processing_state = Processing;
            }
            Processing => {
                self.processing_data.op_cycles += 1;
                self.cycle_execute_op(memory_controller);
            }
        }
    }
    pub fn execute_op(&mut self, memory_controller: &mut MemoryController) {
        if matches!(self.processing_state, LoadingOp) {
            self.process_cycle(memory_controller);
        }
        while !matches!(self.processing_state, LoadingOp) && !(self.processing_data.op_error || self.processing_data.address_error) {
            self.process_cycle(memory_controller);
        }
    }
    fn cycle_execute_op(&mut self, memory_controller: &mut MemoryController) {
        // TODO add per cycle logging with disable option
        match self.processing_data.op {
            Operation::ADC(address_mode) => op_impls::adc(self, memory_controller, &address_mode),
            Operation::BRK => op_impls::brk(self, memory_controller),
            Operation::BRL => op_impls::brl(self, memory_controller),
            Operation::CLC => op_impls::clc(self),
            Operation::CLD => op_impls::cld(self),
            Operation::CLI => op_impls::cli(self),
            Operation::CLV => op_impls::clv(self),
            Operation::COP => op_impls::cop(self, memory_controller),
            Operation::LDA(address_mode) => op_impls::lda(self, memory_controller, &address_mode),
            Operation::PHA => op_impls::pha(self, memory_controller),
            Operation::PHB => op_impls::phb(self, memory_controller),
            Operation::PHD => op_impls::phd(self, memory_controller),
            Operation::PHK => op_impls::phk(self, memory_controller),
            Operation::PHP => op_impls::php(self, memory_controller),
            Operation::PHX => op_impls::phx(self, memory_controller),
            Operation::PHY => op_impls::phy(self, memory_controller),
            Operation::PLA => op_impls::pla(self, memory_controller),
            Operation::PLB => op_impls::plb(self, memory_controller),
            Operation::PLD => op_impls::pld(self, memory_controller),
            Operation::PLP => op_impls::plp(self, memory_controller),
            Operation::PLX => op_impls::plx(self, memory_controller),
            Operation::PLY => op_impls::ply(self, memory_controller),
            Operation::REP => op_impls::rep(self, memory_controller),
            Operation::SEI => op_impls::sei(self),
            Operation::SEP => op_impls::sep(self, memory_controller),
            Operation::STZ(address_mode) => op_impls::stz(self, memory_controller, &address_mode),
            Operation::XCE => xce(self),
            _ => {
                eprintln!(
                    "Unsupported Operation  [Operation::{:?}]",
                    self.processing_data.op
                );
                self.processing_data.op_error = true;
            }
        }
    }
    fn write_byte(
        &mut self,
        memory_controller: &mut MemoryController,
        bank: u8,
        address: u16,
        byte: u8,
    ) {
        self.memory_access_delay = Some(memory_controller.cpu_bus_set_location(bank, address));
        memory_controller.cpu_bus_write(byte);
        self.processing_data.write_count += 1;
    }
    /// Writes a 16 bit word to memory stored in little endian format at bank:address and bank:address+1
    ///
    /// returns a flag indicating if the writing is complete
    fn cycle_write_word(
        &mut self,
        memory_controller: &mut MemoryController,
        bank: u8,
        address: u16,
        word: u16,
    ) -> bool {
        if self.processing_data.write_count == 0 {
            self.write_byte(memory_controller, bank, address, split_bytes(word).0);
            return false;
        } else {
            self.write_byte(
                memory_controller,
                bank,
                address.wrapping_add(1),
                split_bytes(word).1,
            );
            return true;
        }
    }
    fn read_byte(
        &mut self,
        memory_controller: &mut MemoryController,
        bank: u8,
        address: u16,
    ) -> u8 {
        self.memory_access_delay = Some(memory_controller.cpu_bus_set_location(bank, address));
        self.processing_data.read_count += 1;
        memory_controller.cpu_bus_read()
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
        self.processing_data.operand = Some(self.build_operand_from_args(address_mode, is8bit));
    }
    fn build_operand_from_args(&mut self, address_mode: &AddressMode, is8bit: bool) -> u16 {
        let args = &self.processing_data.args;
        match address_mode {
            AddressMode::Absolute => combine_bytes(args[1], args[0]),
            AddressMode::Immediate => {
                let low_byte = args[0];
                let high_byte = if is8bit { 0x0000 } else { args[1] };
                combine_bytes(high_byte, low_byte)
            }

            _ => {
                eprintln!("Unsupported AddressMode::{:?}", address_mode);
                self.processing_data.address_error = true;
                0x0000
            }
        }
    }
    fn load_one_arg(&mut self, memory_controller: &mut MemoryController) {
        let arg_address = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(self.processing_data.args.len() as u16);
        let arg = self.read_byte(memory_controller, self.program_bank, arg_address);
        self.processing_data.args.push(arg);
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
    fn set_processor_status(&mut self, byte: u8) {
        self.processor_status = byte;
        if self.emulation_flag {
            self.processor_status = self.processor_status | INDEX_REG_MASK | ACCUMULATOR_MASK;
        }
        if check_bits_set(self.processor_status, INDEX_REG_MASK) {
            self.x = combine_bytes(0x00, split_bytes(self.x).0);
            self.y = combine_bytes(0x00, split_bytes(self.y).0);
        }
    }
    fn advance_program_counter_with_address_mode(
        &mut self,
        address_mode: &AddressMode,
        is8bit: bool,
    ) {
        let mut op_size = ops::get_op_size(address_mode);
        if is8bit && matches!(address_mode, AddressMode::Immediate) {
            op_size -= 1;
        }
        self.advance_program_counter(op_size);
    }
    fn advance_program_counter(&mut self, op_size: u16) {
        self.processing_state = LoadingOp;
        self.program_counter = self.program_counter.wrapping_add(op_size);
    }
    fn stack_push_byte(&mut self, memory_controller: &mut MemoryController, value: u8) {
        self.write_byte(memory_controller, 0x00, self.stack_pointer, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }
    fn stack_pop_byte(&mut self, memory_controller: &mut MemoryController) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_byte(memory_controller, 0x00, self.stack_pointer)
    }
    /// Pushes bytes to the stack one byte per cycle
    ///
    /// panics if self.processing_data.write_count is >= stack_values.len()
    fn cycle_push_stack(
        &mut self,
        memory_controller: &mut MemoryController,
        stack_values: &Vec<u8>,
    ) -> bool {
        self.stack_push_byte(
            memory_controller,
            (*stack_values)[self.processing_data.write_count as usize],
        );
        self.processing_data.write_count == stack_values.len() as u8
    }
}
