use crate::snes::cpu::instrusctions::{
    AddressMode::{
        Absolute, AbsoluteIndirect, AbsoluteIndirectLong, AbsoluteIndirectX, AbsoluteLong, AbsoluteLongX, AbsoluteX, AbsoluteY, Accumulator, DirectPage, DirectPageIndirect, DirectPageIndirectLong, DirectPageIndirectLongY, DirectPageIndirectX, DirectPageIndirectY, DirectPageX, DirectPageY, Immediate, StackRelative, StackRelativeIndirectY,
    }, Operation::{ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRA, BRK, BRL, BVC, BVS, CLC, CLD, CLI, CLV, CMP, COP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JML, JMP, JSL, JSR, LDA, LDX, LDY, LSR, MVN, MVP, NOP, ORA, PEA, PEI, PER, PHA, PHB, PHD, PHK, PHP, PHX, PHY, PLA, PLB, PLD, PLP, PLX, PLY, REP, ROL, ROR, RTI, RTL, RTS, SBC, SEC, SED, SEI, SEP, STA, STP, STX, STY, STZ, TAX, TAY, TCD, TCS, TDC, TRB, TSB, TSC, TSX, TXA, TXS, TXY, TYA, TYX, WAI, WDM, XBA, XCE},
};

pub enum Operation {
    ADC(OpDetail), // Add with Carry
    AND(OpDetail),              // And Accumulator with Memory
    ASL(OpDetail),              // Arithmetic Shift Left
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

    BIT(OpDetail), // Test Memory Bits against Accumulator
    // Software Interrupts
    // Syntax      Addressing Mode  Opcode  Bytes Cycles   Extra
    BRK, // param     Interrupt     $00     2     7        +1 if e=0
    COP, // param     Interrupt     $02     2     7        +1 if e=0

    // Clear Status Flags
    CLC,
    CLI,
    CLD,
    CLV,

    CMP(OpDetail), // Compare Accumulator with Memory
    CPX(OpDetail), // Compare Index Register X with Memory
    CPY(OpDetail), // Compare Index Register Y with Memory
    DEC(OpDetail), // Decrement
    DEX,
    DEY, // Decrement Index Registers
    EOR(OpDetail), // Exclusive OR Accumulator with Memory
    INC(OpDetail), // Increment
    INX,
    INY,              // Increment Index Registers
    JMP(OpDetail), // Jump
    JML(OpDetail),              // Jump
    JSR(OpDetail),              // Jump to Subroutine
    JSL,              // Jump to Subroutine
    LDA(OpDetail),              // Load Accumulator from Memory
    LDX(OpDetail),              // Load Index Register X from Memory
    LDY(OpDetail),              // Load Index Register Y from Memory
    LSR(OpDetail),              // Logical Shift Right
    MVN,              // Block Move Next
    MVP,              // Block Move Previous
    NOP,              // No Operation
    ORA(OpDetail),              // OR Accumulator with Memory
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
    ROL(OpDetail), // Rotate Left
    ROR(OpDetail), // Rotate Right
    RTI, // Return From Interrupt
    RTS, // Return From Subroutine
    RTL, // Return From Subroutine
    SBC(OpDetail), // Subtract with Borrow from Accumulator
    // Set Status Flags
    //      Name                        Opcode  Bytes  Cycles
    SEC, // Set Carry Flag              $38     1      2
    SEI, // Set Interrupt Disable Flag  $78     1      2
    SED, // Set Decimal Flag            $F8     1      2

    SEP,              // Set Status Bits
    STA(OpDetail), // Store Accumulator to Memory
    STP,              // Stop the Processor
    STX(OpDetail),              // Store Index Register X to Memory
    STY(OpDetail),              // Store Index Register Y to Memory
    STZ(OpDetail), // Store Zero to Memory
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

    TRB(OpDetail), // Test and Reset Memory Bits Against Accumulator
    TSB(OpDetail), // Test and Set Memory Bits Against Accumulator
    WAI, // Wait for Interrupt
    WDM, // Reserved for Future Expansion
    XBA, // Exchange the B and A Accumulators
    XCE, // Exchange Carry and Emulation Bits
}

pub struct OpDetail {
    pub address_mode: AddressMode,
    pub op_size: u16,
}

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

pub fn lookup_opcode(opcode: u8) -> Operation {
    match opcode {
        0x00 => BRK,
        0x01 => ORA(OpDetail { address_mode:DirectPageIndirectX, op_size: 2 }),
        0x02 => COP,
        0x03 => ORA(OpDetail { address_mode:StackRelative, op_size: 2 }),
        0x04 => TSB(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x05 => ORA(OpDetail { address_mode:DirectPage, op_size: 2 }),
        0x06 => ASL(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x07 => ORA(OpDetail { address_mode:DirectPageIndirectLong, op_size: 2 }),
        0x08 => PHP,
        0x09 => ORA(OpDetail { address_mode:Immediate, op_size: 3 }),
        0x0A => ASL(OpDetail { address_mode:Accumulator, op_size: 1 }),
        0x0B => PHD,
        0x0C => TSB(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x0D => ORA(OpDetail { address_mode:Absolute, op_size: 3 }),
        0x0E => ASL(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x0F => ORA(OpDetail { address_mode:AbsoluteLong, op_size: 4 }),
        
        0x10 => BPL,
        0x11 => ORA(OpDetail { address_mode:DirectPageIndirectY, op_size: 2 }),
        0x12 => ORA(OpDetail { address_mode:DirectPageIndirect, op_size: 2 }),
        0x13 => ORA(OpDetail { address_mode:StackRelativeIndirectY, op_size: 2 }),
        0x14 => TRB(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x15 => ORA(OpDetail { address_mode:DirectPageX, op_size: 2 }),
        0x16 => ASL(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x17 => ORA(OpDetail { address_mode:DirectPageIndirectLongY, op_size: 2 }),
        0x18 => CLC,
        0x19 => ORA(OpDetail { address_mode:AbsoluteY, op_size: 3 }),
        0x1A => INC(OpDetail { address_mode: Accumulator, op_size: 1 }),
        0x1B => TCS,
        0x1C => TRB(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x1D => ORA(OpDetail { address_mode:AbsoluteX, op_size: 3 }),
        0x1E => ASL(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x1F => ORA(OpDetail { address_mode:AbsoluteLongX, op_size: 4 }),
        
        0x20 => JSR(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x21 => AND(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0x22 => JSL,
        0x23 => AND(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0x24 => BIT(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x25 => AND(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x26 => ROL(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x27 => AND(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0x28 => PLP,
        0x29 => AND(OpDetail { address_mode: Immediate, op_size: 3 }),
        0x2A => ROL(OpDetail { address_mode: Accumulator, op_size: 1 }),
        0x2B => PLD,
        0x2C => BIT(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x2D => AND(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x2E => ROL(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x2F => AND(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        
        0x30 => BMI,
        0x31 => AND(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0x32 => AND(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0x33 => AND(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0x34 => BIT(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x35 => AND(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x36 => ROL(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x37 => AND(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0x38 => SEC,
        0x39 => AND(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0x3A => DEC(OpDetail { address_mode: Accumulator, op_size: 1 }),
        0x3B => TSC,
        0x3C => BIT(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x3D => AND(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x3E => ROL(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x3F => AND(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),

        0x40 => RTI,
        0x41 => EOR(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0x42 => WDM,
        0x43 => EOR(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0x44 => MVP,
        0x45 => EOR(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x46 => LSR(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x47 => EOR(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0x48 => PHA,
        0x49 => EOR(OpDetail { address_mode: Immediate, op_size: 3 }),
        0x4A => LSR(OpDetail { address_mode: Accumulator, op_size: 1 }),
        0x4B => PHK,
        0x4C => JMP(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x4D => EOR(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x4E => LSR(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x4F => EOR(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),

        0x50 => BVC,
        0x51 => EOR(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0x52 => EOR(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0x53 => EOR(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0x54 => MVN,
        0x55 => EOR(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x56 => LSR(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x57 => EOR(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0x58 => CLI,
        0x59 => EOR(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0x5A => PHY,
        0x5B => TCD,
        0x5D => EOR(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x5C => JML(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        0x5E => LSR(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x5F => EOR(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
        0x60 => RTS,
        0x61 => ADC(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0x62 => PER,
        0x63 => ADC(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0x64 => STZ(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x65 => ADC(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x66 => ROR(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x67 => ADC(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0x68 => PLA,
        0x69 => ADC(OpDetail { address_mode: Immediate, op_size: 3 }),
        0x6A => ROR(OpDetail { address_mode: Accumulator, op_size: 1 }),
        0x6B => RTL,
        0x6C => JMP(OpDetail { address_mode: AbsoluteIndirect, op_size: 3 }),
        0x6D => ADC(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x6E => ROR(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x6F => ADC(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        
        0x70 => BVS,
        0x71 => ADC(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0x72 => ADC(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0x73 => ADC(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0x74 => STZ(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x75 => ADC(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x76 => ROR(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x77 => ADC(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0x78 => SEI,
        0x79 => ADC(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0x7A => PLY,
        0x7B => TDC,
        0x7C => JMP(OpDetail { address_mode: AbsoluteIndirectX, op_size: 3 }),
        0x7D => ADC(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x7E => ROR(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x7F => ADC(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
        0x80 => BRA,
        0x81 => STA(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0x82 => BRL,
        0x83 => STA(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0x84 => STY(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x85 => STA(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x86 => STX(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0x87 => STA(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0x88 => DEY,
        0x89 => BIT(OpDetail { address_mode: Immediate, op_size: 3 }),
        0x8A => TXA,
        0x8B => PHB,
        0x8C => STY(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x8D => STA(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x8E => STX(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x8F => STA(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        
        0x90 => BCC,
        0x91 => STA(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0x92 => STA(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0x93 => STA(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0x94 => STY(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x95 => STA(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0x96 => STX(OpDetail { address_mode: DirectPageY, op_size: 2 }),
        0x97 => STA(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0x98 => TYA,
        0x99 => STA(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0x9A => TXS,
        0x9B => TXY,
        0x9C => STZ(OpDetail { address_mode: Absolute, op_size: 3 }),
        0x9D => STA(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x9E => STZ(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0x9F => STA(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
        0xA0 => LDY(OpDetail { address_mode:Immediate , op_size: 3 }),
        0xA1 => LDA(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0xA2 => LDX(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xA3 => LDA(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0xA4 => LDY(OpDetail { address_mode:DirectPage , op_size: 2 }),
        0xA5 => LDA(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xA6 => LDX(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xA7 => LDA(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0xA8 => TAY,
        0xA9 => LDA(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xAA => TAX,
        0xAB => PLB,
        0xAC => LDY(OpDetail { address_mode:Absolute , op_size: 3 }),
        0xAD => LDA(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xAE => LDX(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xAF => LDA(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        
        0xB0 => BCS,
        0xB1 => LDA(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0xB2 => LDA(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0xB3 => LDA(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0xB4 => LDY(OpDetail { address_mode:DirectPageX , op_size: 2 }),
        0xB5 => LDA(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0xB6 => LDX(OpDetail { address_mode: DirectPageY, op_size: 2 }),
        0xB7 => LDA(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0xB8 => CLV,
        0xB9 => LDA(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0xBA => TSX,
        0xBB => TYX,
        0xBC => LDY(OpDetail { address_mode:AbsoluteX , op_size: 3 }),
        0xBD => LDA(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0xBE => LDX(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0xBF => LDA(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
        0xC0 => CPY(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xC1 => CMP(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0xC2 => REP,
        0xC3 => CMP(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0xC4 => CPY(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xC5 => CMP(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xC6 => DEC(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xC7 => CMP(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0xC8 => INY,
        0xC9 => CMP(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xCA => DEX,
        0xCB => WAI,
        0xCC => CPY(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xCD => CMP(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xCE => DEC(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xCF => CMP(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),
        
        0xD0 => BNE,
        0xD1 => CMP(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0xD2 => CMP(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0xD3 => CMP(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0xD4 => PEI,
        0xD5 => CMP(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0xD6 => DEC(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0xD7 => CMP(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0xD8 => CLD,
        0xD9 => CMP(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0xDA => PHX,
        0xDB => STP,
        0xDC => JML(OpDetail { address_mode: AbsoluteIndirectLong, op_size: 3 }),
        0xDD => CMP(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0xDE => DEC(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0xDF => CMP(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
        0xE0 => CPX(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xE1 => SBC(OpDetail { address_mode: DirectPageIndirectX, op_size: 2 }),
        0xE2 => SEP,
        0xE3 => SBC(OpDetail { address_mode: StackRelative, op_size: 2 }),
        0xE4 => CPX(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xE5 => SBC(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xE6 => INC(OpDetail { address_mode: DirectPage, op_size: 2 }),
        0xE7 => SBC(OpDetail { address_mode: DirectPageIndirectLong, op_size: 2 }),
        0xE8 => INX,
        0xE9 => SBC(OpDetail { address_mode: Immediate, op_size: 3 }),
        0xEA => NOP,
        0xEB => XBA,
        0xEC => CPX(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xED => SBC(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xEE => INC(OpDetail { address_mode: Absolute, op_size: 3 }),
        0xEF => SBC(OpDetail { address_mode: AbsoluteLong, op_size: 4 }),

        0xF0 => BEQ,
        0xF1 => SBC(OpDetail { address_mode: DirectPageIndirectY, op_size: 2 }),
        0xF2 => SBC(OpDetail { address_mode: DirectPageIndirect, op_size: 2 }),
        0xF3 => SBC(OpDetail { address_mode: StackRelativeIndirectY, op_size: 2 }),
        0xF4 => PEA,
        0xF5 => SBC(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0xF6 => INC(OpDetail { address_mode: DirectPageX, op_size: 2 }),
        0xF7 => SBC(OpDetail { address_mode: DirectPageIndirectLongY, op_size: 2 }),
        0xF8 => SED,
        0xF9 => SBC(OpDetail { address_mode: AbsoluteY, op_size: 3 }),
        0xFA => PLX,
        0xFB => XCE,
        0xFC => JSR(OpDetail { address_mode: AbsoluteIndirectX, op_size: 3 }),
        0xFD => SBC(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0xFE => INC(OpDetail { address_mode: AbsoluteX, op_size: 3 }),
        0xFF => SBC(OpDetail { address_mode: AbsoluteLongX, op_size: 4 }),
        
    }
}
