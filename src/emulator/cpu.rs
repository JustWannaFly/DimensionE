use crate::{
    bytes::{build_flag_byte, is_carry16, is_half_carry16},
    emulator::{
        cartridge::Cartridge,
        cpu::instructions::{
            CPURegister::{self, Accumulator, Flags, ProgramCounter},
            Operation, lookup_opcode,
        },
    },
};

mod instructions;

pub struct Register {
    pub low: u8,
    pub high: u8,
}
impl Register {
    pub fn new(high: u8, low: u8) -> Register {
        Register { high, low }
    }
    pub fn set16(&mut self, value: u16) {
        (self.high, self.low) = crate::bytes::split_bytes(value)
    }
    pub fn get16(&self) -> u16 {
        crate::bytes::combine_bytes(self.high, self.low)
    }
}

pub struct CPU<'a> {
    acc_flags: Register,
    reg_bc: Register,
    reg_de: Register,
    reg_hl: Register,
    stack_pointer: Register,
    program_counter: Register,

    cart: &'a Cartridge,
}
impl CPU<'_> {
    pub fn new(cart: &'_ Cartridge) -> CPU<'_> {
        CPU {
            acc_flags: Register::new(0x11, 0b0100_0000),
            reg_bc: Register::new(0x0, 0x13),
            reg_de: Register::new(0x00, 0xD8),
            reg_hl: Register::new(0x01, 0x4D),
            stack_pointer: Register::new(0xFF, 0xFE),
            program_counter: Register::new(0x01, 0x00),

            cart,
        }
    }
    fn set_register8(&mut self, register: CPURegister, value: u8) {
        match register {
            CPURegister::Accumulator => self.acc_flags.high = value,
            CPURegister::Flags => self.acc_flags.low = value,
            CPURegister::B => self.reg_bc.high = value,
            CPURegister::C => self.reg_bc.low = value,
            CPURegister::D => self.reg_de.high = value,
            CPURegister::E => self.reg_de.low = value,
            CPURegister::H => self.reg_hl.high = value,
            CPURegister::L => self.reg_hl.low = value,
            _ => panic!("Tried to set 8 bits to 16 bit register"),
        }
    }
    fn set_register16(&mut self, register: CPURegister, value: u16) {
        match register {
            CPURegister::AF => self.acc_flags.set16(value),
            CPURegister::BC => self.reg_bc.set16(value),
            CPURegister::DE => self.reg_de.set16(value),
            CPURegister::HL => self.reg_hl.set16(value),
            CPURegister::StackPointer => self.stack_pointer.set16(value),
            CPURegister::ProgramCounter => self.program_counter.set16(value),
            _ => panic!("Tried to set 16 bits to 8 bit register"),
        }
    }
    fn get_register_ref(&self, reg: CPURegister) -> &Register {
        match reg {
            CPURegister::AF => &self.acc_flags,
            CPURegister::Accumulator => &self.acc_flags,
            CPURegister::Flags => &self.acc_flags,
            CPURegister::BC => &self.reg_bc,
            CPURegister::B => &self.reg_bc,
            CPURegister::C => &self.reg_bc,
            CPURegister::DE => &self.reg_de,
            CPURegister::D => &self.reg_de,
            CPURegister::E => &self.reg_de,
            CPURegister::HL => &self.reg_hl,
            CPURegister::H => &self.reg_hl,
            CPURegister::L => &self.reg_hl,
            CPURegister::StackPointer => &self.stack_pointer,
            CPURegister::ProgramCounter => &self.program_counter,
        }
    }
    pub fn read_imm16(&self) -> u16 {
        self.cart
            .read16((self.program_counter.get16() + 1) as usize)
    }
    pub fn execute_operation(&mut self) {
        let op_length: u16;
        let _op_time: u64;
        let op_addr = self.program_counter.get16();
        let opcode = self.cart.read8(self.program_counter.get16() as usize);
        println!("opcode: [{:X}]    op_addr: [{:X}]", opcode, op_addr);
        match lookup_opcode(opcode) {
            Operation::NOP => {
                op_length = 1;
                _op_time = 4;
            }
            Operation::LD16Imm { reg } => {
                op_length = 3;
                _op_time = 12;
                self.set_register16(reg, self.read_imm16());
            }
            Operation::ADD16 { reg } => {
                op_length = 1;
                _op_time = 8;
                let hl_val = self.reg_hl.get16();
                let val2 = self.get_register_ref(reg).get16();
                self.reg_hl.set16(hl_val + val2);

                self.acc_flags.low = build_flag_byte(
                    self.acc_flags.low,
                    None,
                    Some(false),
                    Some(is_half_carry16(hl_val, val2)),
                    Some(is_carry16(hl_val, val2)),
                );
            }
            Operation::JP => {
                op_length = 0;
                _op_time = 16;
                self.set_register16(ProgramCounter, self.read_imm16());
            }
        }
        self.program_counter
            .set16(self.program_counter.get16() + op_length);
    }
}
