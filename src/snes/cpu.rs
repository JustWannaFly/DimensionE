use std::{cell::RefCell, rc::Rc};

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
enum ProcessingState {
    LoadingOp,
    Processing,
}

pub struct ProcessingDetails {
    pub op: Operation,
    pub args: Vec<u8>,
    pub operand: Option<u16>,
    pub address: Option<(u8, u16)>,
    pub op_cycles: u8,
    pub write_count: u8,
    pub read_count: u8,
    pub delay_count: u8,
}
pub struct CPU {
    pub accumulator: u16,
    pub x: u16,
    pub y: u16,
    pub direct_page: u16,
    pub stack_pointer: u16, // TODO emulation mode is this 8 bits
    pub processor_status: u8,
    pub data_bank: u8,
    pub program_bank: u8,
    pub program_counter: u16,
    pub emulation_flag: bool,
    pub clock_stop: bool,
    pub ready: bool,
    memory_controller: Rc<RefCell<MemoryController>>,
    memory_access_delay: Option<u8>,
    ticks_since_cycle: u8,
    ops_executed: u32,

    pub processing_state: ProcessingState,
    pub processing_data: ProcessingDetails,
}
impl CPU {
    pub fn new(memory_controller: Rc<RefCell<MemoryController>>) -> CPU {
        CPU {
            accumulator: 0x0000_0000,
            x: 0x0000_0000,
            y: 0x0000_0000,
            direct_page: 0x0000_0000,
            stack_pointer: 0x0000_0000,
            processor_status: ACCUMULATOR_MASK + INDEX_REG_MASK + IRQ_MASK,
            data_bank: 0x0000,
            program_bank: 0x0000,
            program_counter: 0x0000_0000,
            emulation_flag: true,
            clock_stop: false,
            ready: true,
            memory_controller,
            memory_access_delay: None,
            ticks_since_cycle: BASE_CPU_SPEED,
            ops_executed: 0,

            processing_state: LoadingOp,
            processing_data: ProcessingDetails {
                op: Operation::NOP,
                args: vec![],
                operand: None,
                address: None,
                op_cycles: 0,
                write_count: 0,
                read_count: 0,
                delay_count: 0,
            },
        }
    }
    pub fn initialize(&mut self, program_start: u16) {
        // TODO initialize all registers
        self.memory_access_delay = None;
        self.ticks_since_cycle = BASE_CPU_SPEED;

        self.processor_status = ACCUMULATOR_MASK + INDEX_REG_MASK + IRQ_MASK;
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
            address: None,
            op_cycles: 0,
            write_count: 0,
            read_count: 0,
            delay_count: 0,
        };
    }
    pub fn tick(&mut self) {
        if !self.ready {
            // TODO handle NMI or IRQ low interrupt conditions
            return;
        }
        self.ticks_since_cycle += 1;
        let ticks_to_delay = self.memory_access_delay.unwrap_or(BASE_CPU_SPEED);
        if self.ticks_since_cycle >= ticks_to_delay {
            if let Some(_) = self.memory_access_delay {
                self.memory_access_delay = None;
            }
            // println!("");
            self.ticks_since_cycle = 0;
            self.process_cycle();
        }
        // print!("|");
    }
    pub fn process_cycle(&mut self) {
        match &self.processing_state {
            LoadingOp => {
                // println!(
                // "Cycles taken for op: [{}] :: Total Ops: [{}]",
                // self.processing_data.op_cycles, self.ops_executed
                // );
                self.ops_executed = self.ops_executed.wrapping_add(1);
                let op_code = self.read_byte(self.program_bank, self.program_counter);
                self.processing_data = ProcessingDetails {
                    op: lookup_opcode(op_code),
                    args: vec![],
                    operand: None,
                    address: None,
                    op_cycles: 1,
                    write_count: 0,
                    read_count: 0,
                    delay_count: 0,
                };
                println!(
                    "Next instruction: [{:X}:{:X}]:[{:X}:{:?}]",
                    self.program_bank, self.program_counter, op_code, self.processing_data.op
                );
                self.processing_state = Processing;
            }
            Processing => {
                self.processing_data.op_cycles += 1;
                self.cycle_execute_op();
            }
        }
    }
    pub fn execute_op(&mut self) {
        if matches!(self.processing_state, LoadingOp) {
            self.process_cycle();
        }
        while !matches!(self.processing_state, LoadingOp) {
            self.process_cycle();
        }
    }
    fn cycle_execute_op(&mut self) {
        // TODO add per cycle logging with disable option
        match self.processing_data.op {
            Operation::ADC(address_mode) => op_impls::adc(self, &address_mode),
            Operation::AND(address_mode) => op_impls::and(self, &address_mode),
            Operation::ASL(address_mode) => op_impls::asl(self, &address_mode),
            Operation::BCC => op_impls::bcc(self),
            Operation::BCS => op_impls::bcs(self),
            Operation::BEQ => op_impls::beq(self),
            Operation::BIT(address_mode) => op_impls::bit(self, &address_mode),
            Operation::BMI => op_impls::bmi(self),
            Operation::BNE => op_impls::bne(self),
            Operation::BPL => op_impls::bpl(self),
            Operation::BRA => op_impls::bra(self),
            Operation::BRK => op_impls::brk(self),
            Operation::BRL => op_impls::brl(self),
            Operation::BVC => op_impls::bvc(self),
            Operation::BVS => op_impls::bvs(self),
            Operation::CLC => op_impls::clc(self),
            Operation::CLD => op_impls::cld(self),
            Operation::CLI => op_impls::cli(self),
            Operation::CLV => op_impls::clv(self),
            Operation::CMP(address_mode) => op_impls::cmp(self, &address_mode),
            Operation::CPX(address_mode) => op_impls::cpx(self, &address_mode),
            Operation::CPY(address_mode) => op_impls::cpy(self, &address_mode),
            Operation::COP => op_impls::cop(self),
            Operation::DEC(address_mode) => op_impls::dec(self, &address_mode),
            Operation::DEX => op_impls::dex(self),
            Operation::DEY => op_impls::dey(self),
            Operation::EOR(address_mode) => op_impls::eor(self, &address_mode),
            Operation::INC(address_mode) => op_impls::inc(self, &address_mode),
            Operation::INX => op_impls::inx(self),
            Operation::INY => op_impls::iny(self),
            Operation::JML(address_mode) => op_impls::jml(self, &address_mode),
            Operation::JMP(address_mode) => op_impls::jmp(self, &address_mode),
            Operation::JSL => op_impls::jsl(self),
            Operation::JSR(address_mode) => op_impls::jsr(self, &address_mode),
            Operation::LDA(address_mode) => op_impls::lda(self, &address_mode),
            Operation::LDX(address_mode) => op_impls::ldx(self, &address_mode),
            Operation::LDY(address_mode) => op_impls::ldy(self, &address_mode),
            Operation::LSR(address_mode) => op_impls::lsr(self, &address_mode),
            Operation::MVN => op_impls::mvn(self),
            Operation::MVP => op_impls::mvp(self),
            Operation::NOP => op_impls::nop(self),
            Operation::ORA(address_mode) => op_impls::ora(self, &address_mode),
            Operation::PEA => op_impls::pea(self),
            Operation::PEI => op_impls::pei(self),
            Operation::PER => op_impls::per(self),
            Operation::PHA => op_impls::pha(self),
            Operation::PHB => op_impls::phb(self),
            Operation::PHD => op_impls::phd(self),
            Operation::PHK => op_impls::phk(self),
            Operation::PHP => op_impls::php(self),
            Operation::PHX => op_impls::phx(self),
            Operation::PHY => op_impls::phy(self),
            Operation::PLA => op_impls::pla(self),
            Operation::PLB => op_impls::plb(self),
            Operation::PLD => op_impls::pld(self),
            Operation::PLP => op_impls::plp(self),
            Operation::PLX => op_impls::plx(self),
            Operation::PLY => op_impls::ply(self),
            Operation::REP => op_impls::rep(self),
            Operation::ROL(address_mode) => op_impls::rol(self, &address_mode),
            Operation::ROR(address_mode) => op_impls::ror(self, &address_mode),
            Operation::RTI => op_impls::rti(self),
            Operation::RTL => op_impls::rtl(self),
            Operation::RTS => op_impls::rts(self),
            Operation::SBC(address_mode) => op_impls::sbc(self, &address_mode),
            Operation::SEC => op_impls::sec(self),
            Operation::SED => op_impls::sed(self),
            Operation::SEI => op_impls::sei(self),
            Operation::SEP => op_impls::sep(self),
            Operation::STA(address_mode) => op_impls::sta(self, &address_mode),
            Operation::STP => op_impls::stp(self),
            Operation::STX(address_mode) => op_impls::stx(self, &address_mode),
            Operation::STY(address_mode) => op_impls::sty(self, &address_mode),
            Operation::STZ(address_mode) => op_impls::stz(self, &address_mode),
            Operation::TAX => op_impls::tax(self),
            Operation::TAY => op_impls::tay(self),
            Operation::TCD => op_impls::tcd(self),
            Operation::TCS => op_impls::tcs(self),
            Operation::TDC => op_impls::tdc(self),
            Operation::TRB(address_mode) => op_impls::trb(self, &address_mode),
            Operation::TSB(address_mode) => op_impls::tsb(self, &address_mode),
            Operation::TSC => op_impls::tsc(self),
            Operation::TSX => op_impls::tsx(self),
            Operation::TXA => op_impls::txa(self),
            Operation::TXS => op_impls::txs(self),
            Operation::TXY => op_impls::txy(self),
            Operation::TYA => op_impls::tya(self),
            Operation::TYX => op_impls::tyx(self),
            Operation::WAI => op_impls::wai(self),
            Operation::WDM => op_impls::nop(self),
            Operation::XBA => op_impls::xba(self),
            Operation::XCE => xce(self),
        }
    }
    fn write_byte(&mut self, bank: u8, address: u16, byte: u8) {
        let mut memory_controller = self.memory_controller.borrow_mut();
        self.memory_access_delay = Some(memory_controller.cpu_bus_set_location(bank, address));
        memory_controller.cpu_bus_write(byte);
        self.processing_data.write_count += 1;
    }
    /// Writes a 16 bit word to memory stored in little endian format at bank:address and bank:address+1
    ///
    /// returns a flag indicating if the writing is complete
    fn cycle_write_word(&mut self, bank: u8, address: u16, word: u16) {
        if self.processing_data.write_count == 0 {
            self.write_byte(bank, address, split_bytes(word).0);
        } else {
            self.write_byte(bank, address.wrapping_add(1), split_bytes(word).1);
        }
    }
    fn read_byte(&mut self, bank: u8, address: u16) -> u8 {
        let mut memory_controller = self.memory_controller.borrow_mut();
        self.memory_access_delay = Some(memory_controller.cpu_bus_set_location(bank, address));
        self.processing_data.read_count += 1;
        memory_controller.cpu_bus_read()
    }
    fn cycle_address_loading(&mut self, address_mode: &AddressMode) {
        let arg_count: u8 = get_op_size(address_mode) - 1;
        if (self.processing_data.args.len() as u8) < (arg_count) {
            self.load_one_arg();
            if (self.processing_data.args.len() as u8) < (arg_count) {
                return;
            }
        }
        let args = &self.processing_data.args;
        let invalid_arg_count_error = "Unexpected arg count for address mode. This should be impossible and means there is a software bug";
        // TODO verify all address resolutions & timings. I'm tired and confusing myself as I try to check they're correct.
        match address_mode {
            AddressMode::Absolute => {
                self.processing_data.address =
                    Some((self.data_bank, combine_bytes(args[1], args[0])));
                self.processing_data.read_count = 0;
            }
            AddressMode::AbsoluteIndirect => match self.processing_data.args.len() {
                2 => {
                    let address =
                        combine_bytes(self.processing_data.args[1], self.processing_data.args[0]);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let address =
                        combine_bytes(self.processing_data.args[1], self.processing_data.args[0])
                            .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    let resolved_addr = combine_bytes(byte, self.processing_data.args[2]);
                    self.processing_data.address = Some((self.program_bank, resolved_addr));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::AbsoluteIndirectLong => match self.processing_data.args.len() {
                2 => {
                    let byte = self.read_byte(
                        self.program_bank,
                        combine_bytes(self.processing_data.args[1], self.processing_data.args[0]),
                    );
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let byte = self.read_byte(
                        self.program_bank,
                        combine_bytes(self.processing_data.args[1], self.processing_data.args[0])
                            .wrapping_add(1),
                    );
                    self.processing_data.args.push(byte);
                }
                4 => {
                    let byte = self.read_byte(
                        self.program_bank,
                        combine_bytes(self.processing_data.args[1], self.processing_data.args[0])
                            .wrapping_add(2),
                    );
                    self.processing_data.args.push(byte);
                }
                5 => {
                    let resolved_addr =
                        combine_bytes(self.processing_data.args[3], self.processing_data.args[2]);
                    self.processing_data.address =
                        Some((self.processing_data.args[4], resolved_addr));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::AbsoluteIndirectX => match self.processing_data.args.len() {
                2 => {
                    let address = self.x.wrapping_add(combine_bytes(
                        self.processing_data.args[1],
                        self.processing_data.args[0],
                    ));
                    let byte = self.read_byte(self.program_bank, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let address = self
                        .x
                        .wrapping_add(combine_bytes(
                            self.processing_data.args[1],
                            self.processing_data.args[0],
                        ))
                        .wrapping_add(1);
                    let byte = self.read_byte(self.program_bank, address);
                    self.processing_data.args.push(byte);
                }
                4 => {
                    let resolved_addr =
                        combine_bytes(self.processing_data.args[3], self.processing_data.args[2]);
                    self.processing_data.address = Some((self.program_bank, resolved_addr));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::AbsoluteLong => {
                self.processing_data.address = Some((args[2], combine_bytes(args[1], args[0])));
                self.processing_data.read_count = 0;
            }
            AddressMode::AbsoluteLongX => {
                let arg_address = combine_bytes(args[1], args[0]);
                let resolved_address = arg_address.wrapping_add(self.x);
                let dbr_carry = if resolved_address < arg_address { 1 } else { 0 };
                self.processing_data.address =
                    Some((args[2].wrapping_add(dbr_carry), resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::AbsoluteX => {
                let resolved_address = self.x.wrapping_add(combine_bytes(args[1], args[0]));
                self.processing_data.address = Some((self.data_bank, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::AbsoluteY => {
                let resolved_address = self.y.wrapping_add(combine_bytes(args[1], args[0]));
                self.processing_data.address = Some((self.data_bank, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::Accumulator => {
                panic!("Should not try to resolve address for Accumulator addressing")
            }
            AddressMode::DirectPage => {
                if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                    self.processing_data.delay_count += 1;
                    return;
                }
                self.processing_data.delay_count = 0;

                let resolved_address = self
                    .direct_page
                    .wrapping_add(self.processing_data.args[0] as u16);
                self.processing_data.address = Some((0, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::DirectPageIndirect => match self.processing_data.args.len() {
                1 => {
                    if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                        self.processing_data.delay_count += 1;
                        return;
                    }
                    self.processing_data.delay_count = 0;

                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let resolved_address =
                        combine_bytes(self.processing_data.args[2], self.processing_data.args[1]);
                    self.processing_data.address = Some((self.data_bank, resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::DirectPageIndirectLong => match self.processing_data.args.len() {
                1 => {
                    if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                        self.processing_data.delay_count += 1;
                        return;
                    }
                    self.processing_data.delay_count = 0;

                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let address = self
                        .direct_page
                        .wrapping_add(combine_bytes(0, self.processing_data.args[0]))
                        .wrapping_add(2);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                4 => {
                    let resolved_address =
                        combine_bytes(self.processing_data.args[2], self.processing_data.args[1]);
                    self.processing_data.address =
                        Some((self.processing_data.args[3], resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::DirectPageIndirectLongY => match self.processing_data.args.len() {
                1 => {
                    if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                        self.processing_data.delay_count += 1;
                        return;
                    }
                    self.processing_data.delay_count = 0;

                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);

                    let arg_address =
                        combine_bytes(self.processing_data.args[2], self.processing_data.args[1]);

                    let resolved_address = arg_address.wrapping_add(self.y);
                    let bank_carry = if resolved_address < arg_address { 1 } else { 0 };

                    self.processing_data.address =
                        Some((byte.wrapping_add(bank_carry), resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::DirectPageIndirectX => match self.processing_data.args.len() {
                1 => {
                    if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                        self.processing_data.delay_count += 1;
                        return;
                    }
                    self.processing_data.delay_count = 0;

                    let address = self
                        .x
                        .wrapping_add(self.direct_page)
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(self.data_bank, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .x
                        .wrapping_add(self.direct_page)
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(self.data_bank, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let resolved_address =
                        combine_bytes(self.processing_data.args[2], self.processing_data.args[1]);
                    self.processing_data.address = Some((self.data_bank, resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::DirectPageIndirectY => match self.processing_data.args.len() {
                1 => {
                    if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                        self.processing_data.delay_count += 1;
                        return;
                    }
                    self.processing_data.delay_count = 0;

                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .direct_page
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    let resolved_address =
                        combine_bytes(byte, self.processing_data.args[1]).wrapping_add(self.y);
                    self.processing_data.address = Some((self.data_bank, resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
            AddressMode::DirectPageX => {
                if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                    self.processing_data.delay_count += 1;
                    return;
                }
                self.processing_data.delay_count = 0;

                let resolved_address = self
                    .x
                    .wrapping_add(self.direct_page)
                    .wrapping_add(self.processing_data.args[0] as u16);
                self.processing_data.address = Some((0, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::DirectPageY => {
                if self.processing_data.delay_count == 0 && self.direct_page & 0x00FF != 0 {
                    self.processing_data.delay_count += 1;
                    return;
                }
                self.processing_data.delay_count = 0;

                let resolved_address = self
                    .y
                    .wrapping_add(self.direct_page)
                    .wrapping_add(self.processing_data.args[0] as u16);
                self.processing_data.address = Some((0, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::Immediate => {
                panic!("Should not try to resolve address for Immediate addressing")
            }
            AddressMode::StackRelative => {
                let resolved_address = self
                    .stack_pointer
                    .wrapping_add(self.processing_data.args[0] as u16);
                self.processing_data.address = Some((0x00, resolved_address));
                self.processing_data.read_count = 0;
            }
            AddressMode::StackRelativeIndirectY => match self.processing_data.args.len() {
                1 => {
                    let address = self
                        .stack_pointer
                        .wrapping_add(self.processing_data.args[0] as u16);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                2 => {
                    let address = self
                        .stack_pointer
                        .wrapping_add(self.processing_data.args[0] as u16)
                        .wrapping_add(1);
                    let byte = self.read_byte(0x00, address);
                    self.processing_data.args.push(byte);
                }
                3 => {
                    let arg_address =
                        combine_bytes(self.processing_data.args[2], self.processing_data.args[1]);
                    let resolved_address = arg_address.wrapping_add(self.y);
                    let bank_carry = if resolved_address < arg_address { 1 } else { 0 };

                    self.processing_data.address =
                        Some((self.data_bank.wrapping_add(bank_carry), resolved_address));
                    self.processing_data.read_count = 0;
                }
                _ => panic!("{}", invalid_arg_count_error),
            },
        }
    }
    fn cycle_operand_loading(&mut self, address_mode: &AddressMode, is8bit: bool) {
        match address_mode {
            AddressMode::Accumulator => {
                if is8bit {
                    self.processing_data.operand = Some(self.accumulator & 0xFF)
                } else {
                    self.processing_data.operand = Some(self.accumulator)
                }
            }
            AddressMode::Immediate => {
                self.load_one_arg();
                if is8bit {
                    self.processing_data.operand =
                        Some(combine_bytes(0, self.processing_data.args[0]));
                } else if self.processing_data.read_count == 2 {
                    self.processing_data.operand = Some(combine_bytes(
                        self.processing_data.args[1],
                        self.processing_data.args[0],
                    ));
                }
            }
            _ => match self.processing_data.address {
                None => self.cycle_address_loading(address_mode),
                Some(address) => {
                    let byte = self.read_byte(
                        address.0,
                        (address.1).wrapping_add(self.processing_data.read_count as u16),
                    );
                    if is8bit {
                        self.processing_data.operand = Some(byte as u16);
                        self.processing_data.read_count = 0;
                    } else {
                        if self.processing_data.read_count == 1 {
                            self.processing_data.args.push(byte);
                        } else {
                            let arg_count = self.processing_data.args.len();
                            self.processing_data.operand = Some(combine_bytes(
                                byte,
                                self.processing_data.args[arg_count - 1],
                            ));
                            self.processing_data.read_count = 0;
                        }
                    }
                }
            },
        }
    }
    fn load_one_arg(&mut self) {
        let arg_address = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(self.processing_data.args.len() as u16);
        let arg = self.read_byte(self.program_bank, arg_address);
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
        let mut op_size = ops::get_op_size(address_mode) as u16;
        if is8bit && matches!(address_mode, AddressMode::Immediate) {
            op_size -= 1;
        }
        self.advance_program_counter(op_size);
    }
    fn advance_program_counter(&mut self, op_size: u16) {
        self.processing_state = LoadingOp;
        self.program_counter = self.program_counter.wrapping_add(op_size);
    }
    fn stack_push_byte(&mut self, value: u8) {
        self.write_byte(0x00, self.stack_pointer, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }
    fn stack_pop_byte(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_byte(0x00, self.stack_pointer)
    }
    /// Pushes bytes to the stack one byte per cycle
    ///
    /// panics if self.processing_data.write_count is >= stack_values.len()
    fn cycle_push_stack(&mut self, stack_values: &Vec<u8>) -> bool {
        self.stack_push_byte((*stack_values)[self.processing_data.write_count as usize]);
        self.processing_data.write_count == stack_values.len() as u8
    }
}
