
pub enum CPURegister {
    AF,
    Accumulator,
    Flags,
    BC,
    B,
    C,
    DE,
    D,
    E,
    HL,
    H,
    L,
    StackPointer,
    ProgramCounter
}

pub enum Operation {
    NOP,
    ImmLD16{reg: CPURegister},
    LD16{dest: CPURegister, src: CPURegister},
    
    JP,
}

pub fn lookup_opcode(opcode: u8) -> Operation {
    match opcode {
        0x00 => Operation::NOP,
        0x01 => Operation::ImmLD16 { reg: CPURegister::BC },
        0x11 => Operation::ImmLD16 { reg: CPURegister::DE },
        0x21 => Operation::ImmLD16 { reg: CPURegister::HL },
        0x31 => Operation::ImmLD16 { reg: CPURegister::StackPointer },

        0xC3 => Operation::JP,
        _ => panic!("Unsupported opcode: [{:X}]", opcode)
    }
}
