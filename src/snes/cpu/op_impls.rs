use crate::{
    bytes::{check_bits, combine_bytes, split_bytes},
    snes::{
        cpu::{ACCUMULATOR_MASK, CPU, ProcessingState::LoadingOp, ops::AddressMode},
        memory::MemoryController,
    },
};

mod Helpers {
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
        let mut brk_vector_lookup = native_lookup;
        if !cpu.emulation_flag {
            stack_values.push(cpu.program_bank);
            brk_vector_lookup = emulation_lookup;
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

pub fn brk(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = Helpers::stack_jump_helper(cpu, memory_controller, 0xFFE6, 0xFFFE);
    if helper_completed {
        let flag = if cpu.emulation_flag { Some(true) } else { None };
        cpu.update_processor_status(None, None, Some(true), Some(false), flag, None, None, None);
    }
}
pub fn cop(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = Helpers::stack_jump_helper(cpu, memory_controller, 0xFFE4, 0xFFF4);
    if helper_completed {
        cpu.update_processor_status(None, None, Some(true), Some(false), None, None, None, None);
    }
}
pub fn lda(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        cpu.accumulator = operand;
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
pub fn sei(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, Some(true), None, None, None, None, None);
    cpu.advance_program_counter(1);
}
pub fn stz(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits(cpu.processor_status, ACCUMULATOR_MASK);
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

#[cfg(test)]
mod tests {
    use crate::snes::cpu::IRQ_MASK;

    use super::*;
    use std::assert_matches;

    fn set_accumulator8bit(cpu: &mut CPU) {
        cpu.update_processor_status(None, None, None, None, None, Some(true), None, None);
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
        assert!(check_bits(cpu.processor_status, IRQ_MASK));
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

        memory_controller.write(0, 0x0000, 0xA9); // LCA(Immediate)
        memory_controller.write(0, 0x0001, op_value);

        // base speed 2 cycles
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        // accumulator is set to operand value
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.accumulator, op_value as u16);
        assert_eq!(2, cpu.processing_data.op_cycles);
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

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(0, memory_controller.read_byte(0, 0));
        assert_eq!(4, cpu.processing_data.op_cycles);
    }
}
