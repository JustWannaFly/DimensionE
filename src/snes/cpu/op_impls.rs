use crate::{
    bytes::{check_bits, combine_bytes, split_bytes},
    snes::{
        cpu::{ACCUMULATOR_MASK, CPU, ProcessingState::LoadingOp, ops::AddressMode},
        memory::MemoryController,
    },
};

pub fn brk(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let mut stack_values = vec![];
    let mut brk_vector_lookup = 0xFFE6_u16;
    if !cpu.emulation_flag {
        stack_values.push(cpu.program_bank);
        brk_vector_lookup = 0xFFFE;
    }
    let return_address_bytes = split_bytes(cpu.program_counter.wrapping_add(2));
    stack_values.push(return_address_bytes.1);
    stack_values.push(return_address_bytes.0);
    stack_values.push(cpu.processor_status);
    if stack_values.len() as u8 > cpu.processing_data.write_count {
        cpu.cycle_push_stack(memory_controller, &stack_values);
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
                let flag = if cpu.emulation_flag { Some(true) } else { None };
                cpu.update_processor_status(
                    None,
                    None,
                    Some(true),
                    Some(false),
                    flag,
                    None,
                    None,
                    None,
                );
                cpu.processing_state = LoadingOp;
            }
        }
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
            if is8bit {
                cpu.write_byte(memory_controller, 0x00, address, 0x00);
            } else {
                let is_complete = cpu.cycle_write_word(memory_controller, 0, address, 0);
                if is_complete {
                    cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_cpu() -> CPU {
        
    }

    #[test]
    fn lda() {
        
    }
}
