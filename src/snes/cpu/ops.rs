use crate::snes::cpu::ops::{
    AddressMode::{
        Absolute, AbsoluteIndirect, AbsoluteIndirectLong, AbsoluteIndirectX, AbsoluteLong, AbsoluteLongX, AbsoluteX, AbsoluteY, Accumulator, DirectPage, DirectPageIndirect, DirectPageIndirectLong, DirectPageIndirectLongY, DirectPageIndirectX, DirectPageIndirectY, DirectPageX, DirectPageY, Immediate, StackRelative, StackRelativeIndirectY,
    }, Operation::{ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRA, BRK, BRL, BVC, BVS, CLC, CLD, CLI, CLV, CMP, COP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JML, JMP, JSL, JSR, LDA, LDX, LDY, LSR, MVN, MVP, NOP, ORA, PEA, PEI, PER, PHA, PHB, PHD, PHK, PHP, PHX, PHY, PLA, PLB, PLD, PLP, PLX, PLY, REP, ROL, ROR, RTI, RTL, RTS, SBC, SEC, SED, SEI, SEP, STA, STP, STX, STY, STZ, TAX, TAY, TCD, TCS, TDC, TRB, TSB, TSC, TSX, TXA, TXS, TXY, TYA, TYX, WAI, WDM, XBA, XCE},
};

#[derive(Debug)]
pub enum Operation {
    ADC(AddressMode), // Add with Carry
    AND(AddressMode),              // And Accumulator with Memory
    ASL(AddressMode),              // Arithmetic Shift Left
    // Branches, //
    // Syntax         Name                         Condition             Opcode  Bytes Cycles   Extra
    BCC, //  near     Branch if Carry Clear        carry clear - c=0     $90     2     2        +1 if branch taken, +1 if e=1
    BCS, //  near     Branch if Carry Set          carry set - c=1       $B0     2     2        +1 if branch taken, +1 if e=1
    BNE, //  near     Branch if Not Equal          zero clear - z=0      $D0     2     2        +1 if branch taken, +1 if e=1
    BEQ, //  near     Branch if Equal              zero set - z=1        $F0     2     2        +1 if branch taken, +1 if e=1
    BPL, //  near     Branch if Plus               negative clear - n=0  $10     2     2        +1 if branch taken, +1 if e=1
    BMI, //  near     Branch if Minus              negative set - n=1    $30     2     2        +1 if branch taken, +1 if e=1
    BVC, //  near     Branch if Overflow Clear     overflow clear - v=0  $50     2     2        +1 if branch taken, +1 if e=1
    BVS, //  near     Branch if Overflow Set       overflow set - v=1    $70     2     2        +1 if branch taken, +1 if e=1
    BRA, //  near     Branch Always                always                $80     2     3        +1 if e=1
    BRL, //  label    Branch Always Long           always                $82     3     4

    BIT(AddressMode), // Test Memory Bits against Accumulator
    // Software Interrupts
    // Syntax      Addressing Mode  Opcode  Bytes Cycles   Extra
    BRK, // param     Interrupt     $00     2     7        +1 if e=0
    COP, // param     Interrupt     $02     2     7        +1 if e=0

    // Clear Status Flags
    CLC,
    CLI,
    CLD,
    CLV,

    CMP(AddressMode), // Compare Accumulator with Memory
    CPX(AddressMode), // Compare Index Register X with Memory
    CPY(AddressMode), // Compare Index Register Y with Memory
    DEC(AddressMode), // Decrement
    DEX,
    DEY, // Decrement Index Registers
    EOR(AddressMode), // Exclusive OR Accumulator with Memory
    INC(AddressMode), // Increment
    INX,
    INY,              // Increment Index Registers
    JMP(AddressMode), // Jump
    JML(AddressMode),              // Jump
    JSR(AddressMode),              // Jump to Subroutine
    JSL,              // Jump to Subroutine
    LDA(AddressMode),              // Load Accumulator from Memory
    LDX(AddressMode),              // Load Index Register X from Memory
    LDY(AddressMode),              // Load Index Register Y from Memory
    LSR(AddressMode),              // Logical Shift Right
    MVN,              // Block Move Next
    MVP,              // Block Move Previous
    NOP,              // No Operation
    ORA(AddressMode),              // OR Accumulator with Memory
    PEA,              // Push Effective Absolute Address
    PEI,              // Push Effective Indirect Address
    PER,              // Push Effective PC Relative Indirect Address
    // Push to Stack
    //      Name                           Opcode  Bytes Cycles   Extra
    PHA, // Push Accumulator               $48     1     3        +1 if m=0
    PHB, // Push Data Bank                 $8B     1     3
    PHD, // Push Direct Page Register      $0B     1     4
    PHK, // Push Program Bank Register     $4B     1     3
    PHP, // Push Processor Status Register $08     1     3
    PHX, // Push Index Register X          $DA     1     3        +1 if x=0
    PHY, // Push Index Register Y          $5A     1     3        +1 if x=0

    // Pull from Stack
    //      Name                           Opcode  Bytes  Cycles  Extra
    PLA, // Pull Accumulator               $68     1      4       +1 if m=0
    PLB, // Pull Data Bank                 $AB     1      4
    PLD, // Pull Direct Page Register      $2B     1      5
    PLP, // Pull Processor Status Register $28     1      4
    PLX, // Pull Index Register X          $FA     1      4       +1 if x=0
    PLY, // Pull Index Register Y          $7A     1      4       +1 if x=0

    REP, // Reset Status Bits
    ROL(AddressMode), // Rotate Left
    ROR(AddressMode), // Rotate Right
    RTI, // Return From Interrupt
    RTS, // Return From Subroutine
    RTL, // Return From Subroutine
    SBC(AddressMode), // Subtract with Borrow from Accumulator
    // Set Status Flags
    //      Name                        Opcode  Bytes  Cycles
    SEC, // Set Carry Flag              $38     1      2
    SEI, // Set Interrupt Disable Flag  $78     1      2
    SED, // Set Decimal Flag            $F8     1      2

    SEP,              // Set Status Bits
    STA(AddressMode), // Store Accumulator to Memory
    STP,              // Stop the Processor
    STX(AddressMode),              // Store Index Register X to Memory
    STY(AddressMode),              // Store Index Register Y to Memory
    STZ(AddressMode), // Store Zero to Memory
    // Transfer Registers
    // Syntax   Name               Opcode  Bytes  Cycles
    TAX, // Transfer A to X          $AA   1      2
    TAY, // Transfer A to Y          $A8   1      2
    TCD, // Transfer 16 bit A to D   $5B   1      2
    TCS, // Transfer 16 bit A to S   $1B   1      2
    TDC, // Transfer D to 16 bit A   $7B   1      2
    TSC, // Transfer S to 16 bit A   $3B   1      2
    TSX, // Transfer S to X          $BA   1      2
    TXA, // Transfer X to A          $8A   1      2
    TXS, // Transfer X to S          $9A   1      2
    TXY, // Transfer X to Y          $9B   1      2
    TYA, // Transfer Y to A          $98   1      2
    TYX, // Transfer Y to X          $BB   1      2

    TRB(AddressMode), // Test and Reset Memory Bits Against Accumulator
    TSB(AddressMode), // Test and Set Memory Bits Against Accumulator
    WAI, // Wait for Interrupt
    WDM, // Reserved for Future Expansion
    XBA, // Exchange the B and A Accumulators
    XCE, // Exchange Carry and Emulation Bits
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum AddressMode {
    Absolute,
    AbsoluteIndirect,
    AbsoluteIndirectLong,
    AbsoluteIndirectX,
    AbsoluteLong,
    AbsoluteLongX,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    DirectPage,
    DirectPageIndirect,
    DirectPageIndirectLong,
    DirectPageIndirectLongY,
    DirectPageIndirectX,
    DirectPageIndirectY,
    DirectPageX,
    DirectPageY,
    Immediate,
    StackRelative,
    StackRelativeIndirectY,
}
pub fn get_op_size(address_mode: &AddressMode) -> u16 {
    match address_mode {
        Absolute => 3,
        AbsoluteIndirect => 3,
        AbsoluteIndirectLong => 3,
        AbsoluteIndirectX => 3,
        AbsoluteLong => 4,
        AbsoluteLongX => 4,
        AbsoluteX => 3,
        AbsoluteY => 3,
        Accumulator => 1,
        DirectPage => 2,
        DirectPageIndirect => 2,
        DirectPageIndirectLong => 2,
        DirectPageIndirectLongY => 2,
        DirectPageIndirectX => 2,
        DirectPageIndirectY => 2,
        DirectPageX => 2,
        DirectPageY => 2,
        Immediate => 3,
        StackRelative => 2,
        StackRelativeIndirectY => 2,
    }
}


pub fn lookup_opcode(opcode: u8) -> Operation {
    match opcode {
        0x00 => BRK,
        0x01 => ORA(DirectPageIndirectX),
        0x02 => COP,
        0x03 => ORA(StackRelative),
        0x04 => TSB(DirectPage),
        0x05 => ORA(DirectPage),
        0x06 => ASL(DirectPage),
        0x07 => ORA(DirectPageIndirectLong),
        0x08 => PHP,
        0x09 => ORA(Immediate),
        0x0A => ASL(Accumulator),
        0x0B => PHD,
        0x0C => TSB(Absolute),
        0x0D => ORA(Absolute),
        0x0E => ASL(Absolute),
        0x0F => ORA(AbsoluteLong),
        
        0x10 => BPL,
        0x11 => ORA(DirectPageIndirectY),
        0x12 => ORA(DirectPageIndirect),
        0x13 => ORA(StackRelativeIndirectY),
        0x14 => TRB(DirectPage),
        0x15 => ORA(DirectPageX),
        0x16 => ASL(DirectPageX),
        0x17 => ORA(DirectPageIndirectLongY),
        0x18 => CLC,
        0x19 => ORA(AbsoluteY),
        0x1A => INC(Accumulator),
        0x1B => TCS,
        0x1C => TRB(Absolute),
        0x1D => ORA(AbsoluteX),
        0x1E => ASL(AbsoluteX),
        0x1F => ORA(AbsoluteLongX),
        
        0x20 => JSR(Absolute),
        0x21 => AND(DirectPageIndirectX),
        0x22 => JSL,
        0x23 => AND(StackRelative),
        0x24 => BIT(DirectPage),
        0x25 => AND(DirectPage),
        0x26 => ROL(DirectPage),
        0x27 => AND(DirectPageIndirectLong),
        0x28 => PLP,
        0x29 => AND(Immediate),
        0x2A => ROL(Accumulator),
        0x2B => PLD,
        0x2C => BIT(Absolute),
        0x2D => AND(Absolute),
        0x2E => ROL(Absolute),
        0x2F => AND(AbsoluteLong),
        
        0x30 => BMI,
        0x31 => AND(DirectPageIndirectY),
        0x32 => AND(DirectPageIndirect),
        0x33 => AND(StackRelativeIndirectY),
        0x34 => BIT(DirectPageX),
        0x35 => AND(DirectPageX),
        0x36 => ROL(DirectPageX),
        0x37 => AND(DirectPageIndirectLongY),
        0x38 => SEC,
        0x39 => AND(AbsoluteY),
        0x3A => DEC(Accumulator),
        0x3B => TSC,
        0x3C => BIT(AbsoluteX),
        0x3D => AND(AbsoluteX),
        0x3E => ROL(AbsoluteX),
        0x3F => AND(AbsoluteLongX),

        0x40 => RTI,
        0x41 => EOR(DirectPageIndirectX),
        0x42 => WDM,
        0x43 => EOR(StackRelative),
        0x44 => MVP,
        0x45 => EOR(DirectPage),
        0x46 => LSR(DirectPage),
        0x47 => EOR(DirectPageIndirectLong),
        0x48 => PHA,
        0x49 => EOR(Immediate),
        0x4A => LSR(Accumulator),
        0x4B => PHK,
        0x4C => JMP(Absolute),
        0x4D => EOR(Absolute),
        0x4E => LSR(Absolute),
        0x4F => EOR(AbsoluteLong),

        0x50 => BVC,
        0x51 => EOR(DirectPageIndirectY),
        0x52 => EOR(DirectPageIndirect),
        0x53 => EOR(StackRelativeIndirectY),
        0x54 => MVN,
        0x55 => EOR(DirectPageX),
        0x56 => LSR(DirectPageX),
        0x57 => EOR(DirectPageIndirectLongY),
        0x58 => CLI,
        0x59 => EOR(AbsoluteY),
        0x5A => PHY,
        0x5B => TCD,
        0x5D => EOR(AbsoluteX),
        0x5C => JML(AbsoluteLong),
        0x5E => LSR(AbsoluteX),
        0x5F => EOR(AbsoluteLongX),
        
        0x60 => RTS,
        0x61 => ADC(DirectPageIndirectX),
        0x62 => PER,
        0x63 => ADC(StackRelative),
        0x64 => STZ(DirectPage),
        0x65 => ADC(DirectPage),
        0x66 => ROR(DirectPage),
        0x67 => ADC(DirectPageIndirectLong),
        0x68 => PLA,
        0x69 => ADC(Immediate),
        0x6A => ROR(Accumulator),
        0x6B => RTL,
        0x6C => JMP(AbsoluteIndirect),
        0x6D => ADC(Absolute),
        0x6E => ROR(Absolute),
        0x6F => ADC(AbsoluteLong),
        
        0x70 => BVS,
        0x71 => ADC(DirectPageIndirectY),
        0x72 => ADC(DirectPageIndirect),
        0x73 => ADC(StackRelativeIndirectY),
        0x74 => STZ(DirectPageX),
        0x75 => ADC(DirectPageX),
        0x76 => ROR(DirectPageX),
        0x77 => ADC(DirectPageIndirectLongY),
        0x78 => SEI,
        0x79 => ADC(AbsoluteY),
        0x7A => PLY,
        0x7B => TDC,
        0x7C => JMP(AbsoluteIndirectX),
        0x7D => ADC(AbsoluteX),
        0x7E => ROR(AbsoluteX),
        0x7F => ADC(AbsoluteLongX),
        
        0x80 => BRA,
        0x81 => STA(DirectPageIndirectX),
        0x82 => BRL,
        0x83 => STA(StackRelative),
        0x84 => STY(DirectPage),
        0x85 => STA(DirectPage),
        0x86 => STX(DirectPage),
        0x87 => STA(DirectPageIndirectLong),
        0x88 => DEY,
        0x89 => BIT(Immediate),
        0x8A => TXA,
        0x8B => PHB,
        0x8C => STY(Absolute),
        0x8D => STA(Absolute),
        0x8E => STX(Absolute),
        0x8F => STA(AbsoluteLong),
        
        0x90 => BCC,
        0x91 => STA(DirectPageIndirectY),
        0x92 => STA(DirectPageIndirect),
        0x93 => STA(StackRelativeIndirectY),
        0x94 => STY(DirectPageX),
        0x95 => STA(DirectPageX),
        0x96 => STX(DirectPageY),
        0x97 => STA(DirectPageIndirectLongY),
        0x98 => TYA,
        0x99 => STA(AbsoluteY),
        0x9A => TXS,
        0x9B => TXY,
        0x9C => STZ(Absolute),
        0x9D => STA(AbsoluteX),
        0x9E => STZ(AbsoluteX),
        0x9F => STA(AbsoluteLongX),
        
        0xA0 => LDY(Immediate),
        0xA1 => LDA(DirectPageIndirectX),
        0xA2 => LDX(Immediate),
        0xA3 => LDA(StackRelative),
        0xA4 => LDY(DirectPage),
        0xA5 => LDA(DirectPage),
        0xA6 => LDX(DirectPage),
        0xA7 => LDA(DirectPageIndirectLong),
        0xA8 => TAY,
        0xA9 => LDA(Immediate),
        0xAA => TAX,
        0xAB => PLB,
        0xAC => LDY(Absolute),
        0xAD => LDA(Absolute),
        0xAE => LDX(Absolute),
        0xAF => LDA(AbsoluteLong),
        
        0xB0 => BCS,
        0xB1 => LDA(DirectPageIndirectY),
        0xB2 => LDA(DirectPageIndirect),
        0xB3 => LDA(StackRelativeIndirectY),
        0xB4 => LDY(DirectPageX),
        0xB5 => LDA(DirectPageX),
        0xB6 => LDX(DirectPageY),
        0xB7 => LDA(DirectPageIndirectLongY),
        0xB8 => CLV,
        0xB9 => LDA(AbsoluteY),
        0xBA => TSX,
        0xBB => TYX,
        0xBC => LDY(AbsoluteX),
        0xBD => LDA(AbsoluteX),
        0xBE => LDX(AbsoluteY),
        0xBF => LDA(AbsoluteLongX),
        
        0xC0 => CPY(Immediate),
        0xC1 => CMP(DirectPageIndirectX),
        0xC2 => REP,
        0xC3 => CMP(StackRelative),
        0xC4 => CPY(DirectPage),
        0xC5 => CMP(DirectPage),
        0xC6 => DEC(DirectPage),
        0xC7 => CMP(DirectPageIndirectLong),
        0xC8 => INY,
        0xC9 => CMP(Immediate),
        0xCA => DEX,
        0xCB => WAI,
        0xCC => CPY(Absolute),
        0xCD => CMP(Absolute),
        0xCE => DEC(Absolute),
        0xCF => CMP(AbsoluteLong),
        
        0xD0 => BNE,
        0xD1 => CMP(DirectPageIndirectY),
        0xD2 => CMP(DirectPageIndirect),
        0xD3 => CMP(StackRelativeIndirectY),
        0xD4 => PEI,
        0xD5 => CMP(DirectPageX),
        0xD6 => DEC(DirectPageX),
        0xD7 => CMP(DirectPageIndirectLongY),
        0xD8 => CLD,
        0xD9 => CMP(AbsoluteY),
        0xDA => PHX,
        0xDB => STP,
        0xDC => JML(AbsoluteIndirectLong),
        0xDD => CMP(AbsoluteX),
        0xDE => DEC(AbsoluteX),
        0xDF => CMP(AbsoluteLongX),
        
        0xE0 => CPX(Immediate),
        0xE1 => SBC(DirectPageIndirectX),
        0xE2 => SEP,
        0xE3 => SBC(StackRelative),
        0xE4 => CPX(DirectPage),
        0xE5 => SBC(DirectPage),
        0xE6 => INC(DirectPage),
        0xE7 => SBC(DirectPageIndirectLong),
        0xE8 => INX,
        0xE9 => SBC(Immediate),
        0xEA => NOP,
        0xEB => XBA,
        0xEC => CPX(Absolute),
        0xED => SBC(Absolute),
        0xEE => INC(Absolute),
        0xEF => SBC(AbsoluteLong),

        0xF0 => BEQ,
        0xF1 => SBC(DirectPageIndirectY),
        0xF2 => SBC(DirectPageIndirect),
        0xF3 => SBC(StackRelativeIndirectY),
        0xF4 => PEA,
        0xF5 => SBC(DirectPageX),
        0xF6 => INC(DirectPageX),
        0xF7 => SBC(DirectPageIndirectLongY),
        0xF8 => SED,
        0xF9 => SBC(AbsoluteY),
        0xFA => PLX,
        0xFB => XCE,
        0xFC => JSR(AbsoluteIndirectX),
        0xFD => SBC(AbsoluteX),
        0xFE => INC(AbsoluteX),
        0xFF => SBC(AbsoluteLongX),
        
    }
}
