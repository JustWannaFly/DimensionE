use crate::{
    bytes::{check_bits_set, combine_bytes, encode_bdc, get_bdc_value, split_bytes}, snes::{
        cpu::{
            ACCUMULATOR_MASK, CARRY_MASK, CPU, DECIMAL_MASK, INDEX_REG_MASK, ProcessingState::LoadingOp, ops::AddressMode,
        }, memory::MemoryController,
    },
};

mod helpers {
    use crate::{
        bytes::{combine_bytes, split_bytes},
        snes::{
            cpu::{CPU, ProcessingState::LoadingOp},
            memory::MemoryController,
        },
    };

    pub(crate) fn stack_jump_helper(
        cpu: &mut CPU,
        memory_controller: &mut MemoryController,
        native_lookup: u16,
        emulation_lookup: u16,
    ) -> bool {
        let mut stack_values = vec![];
        let mut brk_vector_lookup = emulation_lookup;
        if !cpu.emulation_flag {
            stack_values.push(cpu.program_bank);
            brk_vector_lookup = native_lookup;
        }
        let return_address_bytes = split_bytes(cpu.program_counter.wrapping_add(2));
        stack_values.push(return_address_bytes.1);
        stack_values.push(return_address_bytes.0);
        stack_values.push(cpu.processor_status);
        if stack_values.len() as u8 > cpu.processing_data.write_count {
            cpu.cycle_push_stack(memory_controller, &stack_values);
            false
        } else {
            match cpu.processing_data.read_count {
                0 => {
                    let byte = cpu.read_byte(memory_controller, 0, brk_vector_lookup);
                    cpu.processing_data.args.push(byte);
                    cpu.processing_data.read_count += 1;
                }
                1 => {
                    let byte = cpu.read_byte(memory_controller, 0, brk_vector_lookup + 1);
                    cpu.processing_data.args.push(byte);
                    cpu.processing_data.read_count += 1;
                }
                _ => {
                    cpu.program_bank = 0x00;
                    cpu.program_counter =
                        combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
                    cpu.processing_state = LoadingOp;
                    return true;
                }
            }
            false
        }
    }
}
pub fn adc(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let bit15mask = 0x8000_u16;
        let is_carry_in = check_bits_set(cpu.processor_status, CARRY_MASK);
        let is_bcd = check_bits_set(cpu.processor_status, DECIMAL_MASK);
        let mut sum = if is_carry_in { 1_u32 } else { 0_u32 };
        let mut is_bcd_overflow = false;
        if is_bcd {
            sum = sum + get_bdc_value(cpu.accumulator) as u32 + get_bdc_value(operand) as u32;
            is_bcd_overflow = sum > 9999;
            sum = encode_bdc(sum as u16) as u32;
        } else {
            sum = sum + cpu.accumulator as u32 + operand as u32;
        }
        let mut is_overflow = if is8bit { sum > 0x00FF } else { sum > 0xFFFF };
        let mut is_negative = if is8bit {
            sum & 0x0080 != 0
        } else {
            sum & bit15mask as u32 != 0
        };
        if is_bcd {
            is_negative = false;
            is_overflow = is_bcd_overflow;
        }
        let is_zero = sum == 0;

        if is8bit {
            sum = sum & 0x0000_FFFF;
        }
        cpu.accumulator = sum as u16;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            Some(is_overflow),
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
pub fn brk(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = helpers::stack_jump_helper(cpu, memory_controller, 0xFFE6, 0xFFFE);
    if helper_completed {
        let flag = if cpu.emulation_flag { Some(true) } else { None };
        cpu.update_processor_status(None, None, Some(true), Some(false), flag, None, None, None);
    }
}
pub fn brl(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.args.len() < 2 {
        cpu.load_one_arg(memory_controller);
    } else {
        let shift = combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]) as i16;
        let is_subtract = shift < 0;
        let updated_address = if is_subtract {
            cpu.program_counter
                .wrapping_add(3)
                .wrapping_sub(shift.abs() as u16)
        } else {
            cpu.program_counter
                .wrapping_add(3)
                .wrapping_add(shift as u16)
        };
        cpu.program_counter = updated_address;
        cpu.processing_state = LoadingOp;
    }
}
pub fn clc(cpu: &mut CPU) {
    cpu.update_processor_status(Some(false), None, None, None, None, None, None, None);
    cpu.advance_program_counter(1);
}
pub fn cld(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, None, Some(false), None, None, None, None);
    cpu.advance_program_counter(1);
}
pub fn cli(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, Some(false), None, None, None, None, None);
    cpu.advance_program_counter(1);
}
pub fn clv(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, None, None, None, None, Some(false), None);
    cpu.advance_program_counter(1);
}
pub fn cop(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = helpers::stack_jump_helper(cpu, memory_controller, 0xFFE4, 0xFFF4);
    if helper_completed {
        cpu.update_processor_status(None, None, Some(true), Some(false), None, None, None, None);
    }
}
pub fn lda(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        cpu.accumulator = operand;
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
pub fn pha(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    let bytes = split_bytes(cpu.accumulator);
    let mut required_cycles = 3_u8;
    if is8bit {
        if cpu.processing_data.write_count < 1 {
            cpu.stack_push_byte(memory_controller, bytes.0);
        }
    } else {
        if cpu.processing_data.write_count < 2 {
            cpu.cycle_push_stack(memory_controller, &vec![bytes.0, bytes.1]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
pub fn phb(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.write_count < 1 {
        cpu.stack_push_byte(memory_controller, cpu.data_bank);
    }
    if cpu.processing_data.op_cycles == 3 {
        cpu.advance_program_counter(1);
    }
}
pub fn phd(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let bytes = split_bytes(cpu.direct_page);
    if cpu.processing_data.write_count < 2 {
        cpu.cycle_push_stack(memory_controller, &vec![bytes.0, bytes.1]);
    }
    
    if cpu.processing_data.op_cycles == 4 {
        cpu.advance_program_counter(1);
    }
}
pub fn phx(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let bytes = split_bytes(cpu.x);
    let mut required_cycles = 3_u8;
    if is8bit {
        if cpu.processing_data.write_count < 1 {
            cpu.stack_push_byte(memory_controller, bytes.0);
        }
    } else {
        if cpu.processing_data.write_count < 2 {
            cpu.cycle_push_stack(memory_controller, &vec![bytes.0, bytes.1]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
pub fn phy(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let bytes = split_bytes(cpu.y);
    let mut required_cycles = 3_u8;
    if is8bit {
        if cpu.processing_data.write_count < 1 {
            cpu.stack_push_byte(memory_controller, bytes.0);
        }
    } else {
        if cpu.processing_data.write_count < 2 {
            cpu.cycle_push_stack(memory_controller, &vec![bytes.0, bytes.1]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
pub fn rep(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.args.len() == 0 {
        cpu.load_one_arg(memory_controller);
    } else {
        cpu.processor_status = cpu.processing_data.args[0];
        if cpu.emulation_flag {
            cpu.update_processor_status(None, None, None, None, Some(true), Some(true), None, None);
        }
        cpu.advance_program_counter(2);
    }
}
pub fn sei(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, Some(true), None, None, None, None, None);
    cpu.advance_program_counter(1);
}
pub fn stz(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    match cpu.processing_data.operand {
        None => cpu.cycle_operand_loading(address_mode, memory_controller, is8bit),
        Some(address) => {
            let mut is_complete = false;
            if is8bit {
                cpu.write_byte(memory_controller, 0x00, address, 0x00);
            } else {
                is_complete = cpu.cycle_write_word(memory_controller, 0, address, 0);
            }
            if is_complete || is8bit {
                cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
            }
        }
    }
}
pub fn xce(cpu: &mut CPU) {
    let enter_emulation = check_bits_set(cpu.processor_status, CARRY_MASK);
    let carry = Some(cpu.emulation_flag);
    let set8bit = if enter_emulation { Some(true) } else { None };
    if enter_emulation {
        cpu.stack_pointer = combine_bytes(0x01, split_bytes(cpu.stack_pointer).0)
    }
    cpu.emulation_flag = enter_emulation;
    cpu.update_processor_status(carry, None, None, None, set8bit, set8bit, None, None);
    cpu.advance_program_counter(1);
}

#[cfg(test)]
mod tests {
    use crate::{
        bytes::check_bits_unset,
        snes::cpu::{
            CARRY_MASK, DECIMAL_MASK, INDEX_REG_MASK, IRQ_MASK, OVERFLOW_MASK,
            ProcessingState::LoadingOp,
            ops::{AddressMode::Immediate, Operation},
        },
    };

    use super::*;
    use std::assert_matches;

    fn set_accumulator8bit(cpu: &mut CPU) {
        cpu.update_processor_status(None, None, None, None, None, Some(true), None, None);
    }

    #[test]
    fn adc() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = CARRY_MASK;
        cpu.accumulator = 0x1234;
        memory_controller.write(0x00, 0x0000, 0x69); // ADC(Immediate)
        memory_controller.write(0x00, 0x0001, 0xFF);
        memory_controller.write(0x00, 0x0002, 0xFF);
        let expected_sum = 0x1234_u16.wrapping_add(0xFFFF).wrapping_add(1);

        // Immediate Mode 16 bit = 3 cycles
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::ADC(Immediate));
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(expected_sum, cpu.accumulator);
        assert!(check_bits_set(cpu.processor_status, OVERFLOW_MASK));
    }
    #[test]
    fn brk() {
        let bank = 0x12_u8;
        let address = 0x3456_u16;

        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(address);
        cpu.program_bank = bank;
        cpu.emulation_flag = false;
        cpu.stack_pointer = 0x0005;
        memory_controller.write(bank, address, 0x00); // BRK

        // non-emulation speed 8 cycles;
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BRK);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 8);
        assert_eq!(cpu.stack_pop_byte(&mut memory_controller), 0);
        assert_eq!(
            cpu.stack_pop_byte(&mut memory_controller),
            split_bytes(address).0 + 2
        );
        assert_eq!(
            cpu.stack_pop_byte(&mut memory_controller),
            split_bytes(address).1
        );
        assert_eq!(cpu.stack_pop_byte(&mut memory_controller), bank);
        assert_eq!(cpu.program_bank, 0x00);
        assert_eq!(cpu.program_counter, 0x0000);
        assert!(check_bits_set(cpu.processor_status, IRQ_MASK));
    }
    #[test]
    fn brl() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0x82); // BRL
        memory_controller.write(0x00, 0x0001, 0xFC);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BRL);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 4);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn clc() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0x18); // CLC
        cpu.processor_status = CARRY_MASK;
        assert!(check_bits_set(cpu.processor_status, CARRY_MASK));

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::CLC);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(2, cpu.processing_data.op_cycles);
        assert!(check_bits_unset(cpu.processor_status, CARRY_MASK));
    }
    #[test]
    fn cld() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0xD8); // CLD
        cpu.processor_status = DECIMAL_MASK;
        assert!(check_bits_set(cpu.processor_status, DECIMAL_MASK));

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::CLD);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(2, cpu.processing_data.op_cycles);
        assert!(check_bits_unset(cpu.processor_status, DECIMAL_MASK));
    }
    #[test]
    fn cli() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0x58); // CLI
        cpu.processor_status = IRQ_MASK;
        assert!(check_bits_set(cpu.processor_status, IRQ_MASK));

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::CLI);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(2, cpu.processing_data.op_cycles);
        assert!(check_bits_unset(cpu.processor_status, IRQ_MASK));
    }
    #[test]
    fn clv() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0xB8); // CLV
        cpu.processor_status = OVERFLOW_MASK;
        assert!(check_bits_set(cpu.processor_status, OVERFLOW_MASK));

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::CLV);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(2, cpu.processing_data.op_cycles);
        assert!(check_bits_unset(cpu.processor_status, OVERFLOW_MASK));
    }
    #[test]
    fn cop() {
        let bank = 0x12_u8;
        let address = 0x3456_u16;

        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(address);
        cpu.program_bank = bank;
        cpu.emulation_flag = false;
        cpu.stack_pointer = 0x0005;
        memory_controller.write(bank, address, 0x02); // COP

        // non-emulation speed 8 cycles;
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::COP);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 8);
        assert_eq!(cpu.stack_pop_byte(&mut memory_controller), 0);
        assert_eq!(
            cpu.stack_pop_byte(&mut memory_controller),
            split_bytes(address).0 + 2
        );
        assert_eq!(
            cpu.stack_pop_byte(&mut memory_controller),
            split_bytes(address).1
        );
        assert_eq!(cpu.stack_pop_byte(&mut memory_controller), bank);
        assert_eq!(cpu.program_bank, 0x00);
        assert_eq!(cpu.program_counter, 0x0000);
    }
    #[test]
    fn lda() {
        let op_value: u8 = 0x12;
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();

        assert_eq!(cpu.accumulator, 0x0000);
        assert_matches!(cpu.processing_state, LoadingOp);

        cpu.initialize(0);
        set_accumulator8bit(&mut cpu);

        memory_controller.write(0, 0x0000, 0xA9); // LDA(Immediate)
        memory_controller.write(0, 0x0001, op_value);

        // base speed 2 cycles
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        // accumulator is set to operand value
        assert_matches!(
            cpu.processing_data.op,
            Operation::LDA(AddressMode::Immediate)
        );
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.accumulator, op_value as u16);
        assert_eq!(2, cpu.processing_data.op_cycles);
    }
    #[test]
    fn rep() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0xC2); // SEI
        memory_controller.write(0x00, 0x0001, 0xFF);
        memory_controller.write(0x00, 0x0002, 0xC2); // SEI
        memory_controller.write(0x00, 0x0003, 0x00);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::REP);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(0xFF, cpu.processor_status);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::REP);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(0b0011_0000, cpu.processor_status);
    }
    #[test]
    fn sei() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        assert_eq!(0b0000_0000, cpu.processor_status);
        memory_controller.write(0x00, 0x0000, 0x78); // SEI

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(0b0000_0100, cpu.processor_status);
        assert_eq!(2, cpu.processing_data.op_cycles);
    }
    #[test]
    fn stz() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        set_accumulator8bit(&mut cpu);
        memory_controller.write(0x00, 0x0000, 0x9C); // STZ(Absolute)
        memory_controller.write(0x00, 0x0001, 0x00);
        memory_controller.write(0x00, 0x0002, 0x00);

        // Absolute addressing speed 4 cycles
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(
            cpu.processing_data.op,
            Operation::STZ(AddressMode::Absolute)
        );
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(0, memory_controller.read_byte(0, 0));
        assert_eq!(4, cpu.processing_data.op_cycles);
    }
    #[test]
    fn xce() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();

        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0xFB);
        memory_controller.write(0x00, 0x0001, 0xFB);
        cpu.processor_status = 0b0000_0000;

        // exchange 1 time to set not emulation mode
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::XCE);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 2);
        assert!(!cpu.emulation_flag);
        assert!(check_bits_set(cpu.processor_status, CARRY_MASK));
        assert!(check_bits_unset(cpu.processor_status, ACCUMULATOR_MASK));
        assert!(check_bits_unset(cpu.processor_status, INDEX_REG_MASK));

        // exchange again to enter emulation mode
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::XCE);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 2);
        assert!(cpu.emulation_flag);
        assert!(check_bits_unset(cpu.processor_status, CARRY_MASK));
        assert!(check_bits_set(cpu.processor_status, ACCUMULATOR_MASK));
        assert!(check_bits_set(cpu.processor_status, INDEX_REG_MASK));
    }
}
