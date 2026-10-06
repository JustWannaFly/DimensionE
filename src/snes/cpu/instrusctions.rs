use crate::snes::cpu::instrusctions::{
    AddressMode::{
        Absolute, AbsoluteIndirect, AbsoluteIndirectX, AbsoluteLong, AbsoluteLongX, AbsoluteX, AbsoluteY, DirectPage, DirectPageX,
    }, Operation::{ADC, CLC, CLD, CLI, CLV, JMP, NOP, SEI, STA, STZ, XCE},
};

pub enum Operation {
    ADC(OpDetail), // Add with Carry
    AND,              // And Accumulator with Memory
    ASL,              // Arithmetic Shift Left
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

    BIT, // Test Memory Bits against Accumulator
    // Software Interrupts
    // Syntax      Addressing Mode  Opcode  Bytes Cycles   Extra
    BRK, // param     Interrupt     $00     2     7        +1 if e=0
    COP, // param     Interrupt     $02     2     7        +1 if e=0

    // Clear Status Flags
    CLC,
    CLI,
    CLD,
    CLV,

    CMP, // Compare Accumulator with Memory
    CPX, // Compare Index Register X with Memory
    CPY, // Compare Index Register Y with Memory
    DEC, // Decrement
    DEX,
    DEY, // Decrement Index Registers
    EOR, // Exclusive OR Accumulator with Memory
    INC, // Increment
    INX,
    INY,              // Increment Index Registers
    JMP(OpDetail), // Jump
    JML,              // Jump
    JSR,              // Jump to Subroutine
    JSL,              // Jump to Subroutine
    LDA,              // Load Accumulator from Memory
    LDX,              // Load Index Register X from Memory
    LDY,              // Load Index Register Y from Memory
    LSR,              // Logical Shift Right
    MVN,              // Block Move Next
    MVP,              // Block Move Previous
    NOP,              // No Operation
    ORA,              // OR Accumulator with Memory
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
    ROL, // Rotate Left
    ROR, // Rotate Right
    RTI, // Return From Interrupt
    RTS, // Return From Subroutine
    RTL, // Return From Subroutine
    SBC, // Subtract with Borrow from Accumulator
    // Set Status Flags
    //      Name                        Opcode  Bytes  Cycles
    SEC, // Set Carry Flag              $38     1      2
    SEI, // Set Interrupt Disable Flag  $78     1      2
    SED, // Set Decimal Flag            $F8     1      2

    SEP,              // Set Status Bits
    STA(OpDetail), // Store Accumulator to Memory
    STP,              // Stop the Processor
    STX,              // Store Index Register X to Memory
    STY,              // Store Index Register Y to Memory
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

    TRB, // Test and Reset Memory Bits Against Accumulator
    TSB, // Test and Set Memory Bits Against Accumulator
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
    DirectPage,
    DirectPageX,
}

pub fn lookup_opcode(opcode: u8) -> Operation {
    match opcode {
        0x00 => NOP,
        
        0x18 => CLC,
        
        0x58 => CLI,
        0x5C => JMP(OpDetail{address_mode: AbsoluteLong, op_size: 4}),
        0x4C => JMP(OpDetail { address_mode: Absolute, op_size: 3}),
        
        0x64 => STZ(OpDetail{address_mode: DirectPage, op_size: 2}),
        0x65 => ADC(OpDetail{address_mode: DirectPage, op_size: 2}),
        0x6C => JMP(OpDetail { address_mode: AbsoluteIndirect, op_size: 3}),
        0x6D => ADC(OpDetail{address_mode: Absolute, op_size: 3}),
        0x6F => ADC(OpDetail{address_mode: AbsoluteLong, op_size: 4}),
        
        0x74 => STZ(OpDetail{address_mode: DirectPageX, op_size: 2}),
        0x78 => SEI,
        0x79 => ADC(OpDetail{address_mode: AbsoluteY, op_size: 3}),
        0x7C => JMP(OpDetail { address_mode: AbsoluteIndirectX, op_size: 3 }),
        0x7D => ADC(OpDetail{address_mode: AbsoluteX, op_size: 3}),
        0x7F => ADC(OpDetail{address_mode: AbsoluteLongX, op_size: 4}),
        
        0x85 => STA(OpDetail{address_mode: DirectPage, op_size: 2}),
        0x8D => STA(OpDetail{address_mode: Absolute, op_size: 3}),
        0x8F => STA(OpDetail{address_mode: AbsoluteLong, op_size: 4}),
        
        0x9C => STZ(OpDetail{address_mode: Absolute, op_size: 3}),
        0x9D => STA(OpDetail{address_mode: AbsoluteX, op_size: 3}),
        0x9E => STZ(OpDetail{address_mode: AbsoluteX, op_size: 3}),
        0x9F => STA(OpDetail{address_mode: AbsoluteLongX, op_size: 4}),
        
        0xB8 => CLV,
        
        0xD8 => CLD,
        0xDC => JMP(OpDetail { address_mode: AbsoluteIndirectLong, op_size: 3 }),
        
        0xFB => XCE,
        _ => panic!("Unsupported opcode: [{:X}]", opcode),
    }
}
