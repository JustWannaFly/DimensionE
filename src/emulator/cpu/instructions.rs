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
    ProgramCounter,
}

pub enum Operation {
    NOP,
    LD16Imm { reg: CPURegister },
    ADD16 { reg: CPURegister },
    JP,
}

pub fn lookup_opcode(opcode: u8) -> Operation {
    match opcode {
        0x00 => Operation::NOP,
        0x01 => Operation::LD16Imm {
            reg: CPURegister::BC,
        },
        0x11 => Operation::LD16Imm {
            reg: CPURegister::DE,
        },
        0x19 => Operation::ADD16 {
            reg: CPURegister::DE,
        },
        0x21 => Operation::LD16Imm {
            reg: CPURegister::HL,
        },
        0x31 => Operation::LD16Imm {
            reg: CPURegister::StackPointer,
        },

        0xC3 => Operation::JP,
        _ => panic!("Unsupported opcode: [{:X}]", opcode),
    }
}
