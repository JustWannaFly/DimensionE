use crate::{
    bytes::{check_bits_set, combine_bytes, encode_bdc, get_bdc_value, split_bytes},
    snes::{
        cpu::{
            ACCUMULATOR_MASK, CARRY_MASK, CPU, DECIMAL_MASK, INDEX_REG_MASK, IRQ_MASK,
            NEGATIVE_MASK, OVERFLOW_MASK,
            ProcessingState::LoadingOp,
            ZERO_MASK,
            op_impls::helpers::{branch_if_clear, branch_if_set, branch_near_helper},
            ops::AddressMode::{self, Immediate},
        },
        memory::MemoryController,
    },
};

mod helpers {
    use crate::{
        bytes::{check_bits_set, check_bits_unset, combine_bytes, split_bytes},
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
                }
                1 => {
                    let byte = cpu.read_byte(memory_controller, 0, brk_vector_lookup + 1);
                    cpu.processing_data.args.push(byte);
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
    pub fn branch_near_helper(
        cpu: &mut CPU,
        memory_controller: &mut MemoryController,
        take_branch: bool,
    ) {
        if take_branch {
            let mut last_cycle = 3_u8;
            if cpu.processing_data.read_count == 0 {
                cpu.load_one_arg(memory_controller);
            } else {
                let offset = cpu.processing_data.args[0] as i8;

                let mut destination_address = cpu.program_counter.wrapping_add(2);
                let did_wrap: bool;
                if offset < 0 {
                    destination_address =
                        destination_address.wrapping_sub(offset.unsigned_abs() as u16);
                    did_wrap = destination_address > cpu.program_counter;
                } else {
                    destination_address = destination_address.wrapping_add(offset as u16);
                    did_wrap = destination_address < cpu.program_counter;
                };
                // if wraps and in emulation mode add a 1 cycle penalty
                if cpu.emulation_flag && did_wrap {
                    last_cycle += 1;
                }
                if cpu.processing_data.op_cycles == last_cycle {
                    cpu.program_counter = destination_address;
                    cpu.processing_state = LoadingOp;
                }
            }
        } else {
            cpu.advance_program_counter(2);
        }
    }
    pub(crate) fn branch_if_set(
        cpu: &mut CPU,
        memory_controller: &mut MemoryController,
        bit_mask: u8,
    ) {
        branch_near_helper(
            cpu,
            memory_controller,
            check_bits_set(cpu.processor_status, bit_mask),
        );
    }
    pub(crate) fn branch_if_clear(
        cpu: &mut CPU,
        memory_controller: &mut MemoryController,
        bit_mask: u8,
    ) {
        branch_near_helper(
            cpu,
            memory_controller,
            check_bits_unset(cpu.processor_status, bit_mask),
        );
    }
}
/// Add with Carry
pub fn adc(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
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
            sum & 0x8000 != 0
        };
        if is_bcd {
            is_negative = false;
            is_overflow = is_bcd_overflow;
        }
        let is_zero = sum == 0;

        if is8bit {
            sum = sum & 0x0000_00FF;
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
/// AND
pub fn and(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        if is8bit {
            high_bit_mask = 0x80_u16;
        } else {
            high_bit_mask = 0x8000;
        }
        cpu.accumulator = cpu.accumulator & operand;
        let is_negative = cpu.accumulator & high_bit_mask != 0;
        let is_zero = cpu.accumulator == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Arithmetic Shift Left
pub fn asl(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let result;
        if is8bit {
            high_bit_mask = 0x80_u16;
            result = (operand << 1) & 0xFF;
        } else {
            high_bit_mask = 0x8000_u16;
            result = operand << 1;
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit
            || cpu.processing_data.write_count == 2
            || matches!(cpu.processing_data.address, None)
        {
            let is_zero = result == 0;
            let is_carry = operand & high_bit_mask != 0;
            let is_negative = result & high_bit_mask != 0;
            cpu.update_processor_status(
                Some(is_carry),
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(is_negative),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Branch Carry Clear
pub fn bcc(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_clear(cpu, memory_controller, CARRY_MASK);
}
/// Branch Carry Set
pub fn bcs(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_set(cpu, memory_controller, CARRY_MASK);
}
/// Branch Zero Set
pub fn beq(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_set(cpu, memory_controller, ZERO_MASK);
}
/// BIT
pub fn bit(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let second_bit_mask;
        if is8bit {
            high_bit_mask = 0x80_u16;
            second_bit_mask = 0x40_u16;
        } else {
            high_bit_mask = 0x8000;
            second_bit_mask = 0x4000;
        }
        let result = cpu.accumulator & operand;
        let mut is_negative = Some(result & high_bit_mask != 0);
        let mut is_overflow = Some(result & second_bit_mask != 0);
        let is_zero = Some(result == 0);
        if matches!(address_mode, Immediate) {
            is_negative = None;
            is_overflow = None;
        }
        cpu.update_processor_status(
            None,
            is_zero,
            None,
            None,
            None,
            None,
            is_overflow,
            is_negative,
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Branch Negative Set
pub fn bmi(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_set(cpu, memory_controller, NEGATIVE_MASK);
}
/// Branch Zero Clear
pub fn bne(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_clear(cpu, memory_controller, ZERO_MASK);
}
/// Branch Negative Clear
pub fn bpl(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_clear(cpu, memory_controller, NEGATIVE_MASK);
}
/// Branch Always
pub fn bra(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_near_helper(cpu, memory_controller, true)
}
/// Software Interrupt
pub fn brk(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = helpers::stack_jump_helper(cpu, memory_controller, 0xFFE6, 0xFFFE);
    if helper_completed {
        let flag = if cpu.emulation_flag { Some(true) } else { None };
        cpu.update_processor_status(None, None, Some(true), Some(false), flag, None, None, None);
    }
}
/// Branch Always Long
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
/// Branch Overflow Clear
pub fn bvc(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_clear(cpu, memory_controller, OVERFLOW_MASK);
}
/// Branch Overflow Set
pub fn bvs(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    branch_if_set(cpu, memory_controller, OVERFLOW_MASK);
}
/// Clear Carry Flag
pub fn clc(cpu: &mut CPU) {
    cpu.update_processor_status(Some(false), None, None, None, None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Clear Decimal Flag
pub fn cld(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, None, Some(false), None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Clear Interrupt Flag
pub fn cli(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, Some(false), None, None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Clear Overflow Flag
pub fn clv(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, None, None, None, None, Some(false), None);
    cpu.advance_program_counter(1);
}
/// Compare with Accumulator
pub fn cmp(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let is_carry;
        let is_zero;
        let is_negative;
        if is8bit {
            let result = (cpu.accumulator as u8).wrapping_sub(operand as u8);
            is_negative = result & 0x80 != 0;
            is_zero = result == 0;
            is_carry = cpu.accumulator as u8 >= operand as u8;
        } else {
            let result = cpu.accumulator.wrapping_sub(operand);
            is_negative = result & 0x8000 != 0;
            is_zero = result == 0;
            is_carry = cpu.accumulator >= operand;
        }
        cpu.update_processor_status(
            Some(is_carry),
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Compare with X
pub fn cpx(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let is_carry;
        let is_zero;
        let is_negative;
        if is8bit {
            let result = (cpu.x as u8).wrapping_sub(operand as u8);
            is_negative = result & 0x80 != 0;
            is_zero = result == 0;
            is_carry = cpu.x as u8 >= operand as u8;
        } else {
            let result = cpu.x.wrapping_sub(operand);
            is_negative = result & 0x8000 != 0;
            is_zero = result == 0;
            is_carry = cpu.x >= operand;
        }
        cpu.update_processor_status(
            Some(is_carry),
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Compare with Y
pub fn cpy(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let is_carry;
        let is_zero;
        let is_negative;
        if is8bit {
            let result = (cpu.y as u8).wrapping_sub(operand as u8);
            is_negative = result & 0x80 != 0;
            is_zero = result == 0;
            is_carry = cpu.y as u8 >= operand as u8;
        } else {
            let result = cpu.y.wrapping_sub(operand);
            is_negative = result & 0x8000 != 0;
            is_zero = result == 0;
            is_carry = cpu.y >= operand;
        }
        cpu.update_processor_status(
            Some(is_carry),
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Co-Processor Interrupt
pub fn cop(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let helper_completed = helpers::stack_jump_helper(cpu, memory_controller, 0xFFE4, 0xFFF4);
    if helper_completed {
        cpu.update_processor_status(None, None, Some(true), Some(false), None, None, None, None);
    }
}
/// Decrement
pub fn dec(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let result;
        if is8bit {
            high_bit_mask = 0x80_u16;
            result = (operand as u8).wrapping_sub(1) as u16;
        } else {
            high_bit_mask = 0x8000;
            result = operand.wrapping_sub(1);
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
                cpu.advance_program_counter(1);
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit || cpu.processing_data.write_count == 2 {
            let is_negative = result & high_bit_mask != 0;
            let is_zero = result == 0;
            cpu.update_processor_status(
                None,
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(is_negative),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Decrement X
pub fn dex(cpu: &mut CPU) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let high_bit_mask;
    let result;
    if is8bit {
        high_bit_mask = 0x80_u16;
        result = (cpu.x as u8).wrapping_sub(1) as u16;
    } else {
        high_bit_mask = 0x8000;
        result = cpu.x.wrapping_sub(1);
    }
    cpu.x = result;

    let is_negative = result & high_bit_mask != 0;
    let is_zero = result == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Decrement Y
pub fn dey(cpu: &mut CPU) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let high_bit_mask;
    let result;
    if is8bit {
        high_bit_mask = 0x80_u16;
        result = (cpu.y as u8).wrapping_sub(1) as u16;
    } else {
        high_bit_mask = 0x8000;
        result = cpu.y.wrapping_sub(1);
    }
    cpu.y = result;

    let is_negative = result & high_bit_mask != 0;
    let is_zero = result == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Exclusive Or
pub fn eor(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let result;
        if is8bit {
            high_bit_mask = 0x80_u16;
            result = (operand as u8 ^ cpu.accumulator as u8) as u16;
        } else {
            high_bit_mask = 0x8000;
            result = operand ^ cpu.accumulator;
        }
        cpu.accumulator = result;
        let is_negative = result & high_bit_mask != 0;
        let is_zero = result == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Increment
pub fn inc(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let result;
        if is8bit {
            high_bit_mask = 0x80_u16;
            result = (operand as u8).wrapping_add(1) as u16;
        } else {
            high_bit_mask = 0x8000;
            result = operand.wrapping_add(1);
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
                cpu.advance_program_counter(1);
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit || cpu.processing_data.write_count == 2 {
            let is_negative = result & high_bit_mask != 0;
            let is_zero = result == 0;
            cpu.update_processor_status(
                None,
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(is_negative),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Increment X
pub fn inx(cpu: &mut CPU) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let high_bit_mask;
    let result;
    if is8bit {
        high_bit_mask = 0x80_u16;
        result = (cpu.x as u8).wrapping_add(1) as u16;
    } else {
        high_bit_mask = 0x8000;
        result = cpu.x.wrapping_add(1);
    }
    cpu.x = result;

    let is_negative = result & high_bit_mask != 0;
    let is_zero = result == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Increment Y
pub fn iny(cpu: &mut CPU) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let high_bit_mask;
    let result;
    if is8bit {
        high_bit_mask = 0x80_u16;
        result = (cpu.y as u8).wrapping_add(1) as u16;
    } else {
        high_bit_mask = 0x8000;
        result = cpu.y.wrapping_add(1);
    }
    cpu.y = result;

    let is_negative = result & high_bit_mask != 0;
    let is_zero = result == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Jump Long
pub fn jml(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    if let None = cpu.processing_data.address {
        cpu.cycle_address_loading(address_mode, memory_controller);
    }
    if let Some(address) = cpu.processing_data.address {
        cpu.program_bank = address.0;
        cpu.program_counter = address.1;
        cpu.processing_state = LoadingOp;
    }
}
/// Jump
pub fn jmp(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    if let None = cpu.processing_data.address {
        cpu.cycle_address_loading(address_mode, memory_controller);
    }
    if let Some(address) = cpu.processing_data.address {
        cpu.program_counter = address.1;
        cpu.processing_state = LoadingOp;
    }
}
/// Jump to Subroutine Long
pub fn jsl(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if let None = cpu.processing_data.address {
        cpu.cycle_address_loading(&AddressMode::AbsoluteLong, memory_controller);
        return;
    }
    if cpu.processing_data.write_count < 3 {
        let pc_bytes = split_bytes(cpu.program_counter.wrapping_add(4));
        cpu.cycle_push_stack(
            memory_controller,
            &vec![cpu.program_bank, pc_bytes.0, pc_bytes.1],
        );
        return;
    }
    cpu.program_counter = cpu.processing_data.address.unwrap().1;
    cpu.processing_state = LoadingOp;
}
/// Jump to Subroutine
pub fn jsr(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    if let None = cpu.processing_data.address {
        cpu.cycle_address_loading(address_mode, memory_controller);
        return;
    }
    if cpu.processing_data.write_count < 2 {
        let pc_bytes = split_bytes(cpu.program_counter.wrapping_add(2));
        cpu.cycle_push_stack(memory_controller, &vec![pc_bytes.0, pc_bytes.1]);
        return;
    }
    cpu.program_counter = cpu.processing_data.address.unwrap().1;
    cpu.processing_state = LoadingOp;
}
/// Load Accumulator
pub fn lda(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        cpu.accumulator = operand;
        let neg_mask = if is8bit { 0x80_u16 } else { 0x8000 };
        let is_negative = cpu.y & neg_mask != 0;
        let is_zero = cpu.accumulator == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Load X
pub fn ldx(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        cpu.x = operand;
        let neg_mask = if is8bit { 0x80_u16 } else { 0x8000 };
        let is_negative = cpu.x & neg_mask != 0;
        let is_zero = cpu.x == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Load Y
pub fn ldy(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        cpu.y = operand;
        let neg_mask = if is8bit { 0x80_u16 } else { 0x8000 };
        let is_negative = cpu.y & neg_mask != 0;
        let is_zero = cpu.y == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Logical Shift Right
pub fn lsr(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let result;
        if is8bit {
            result = (operand as u8 >> 1) as u16;
        } else {
            result = operand >> 1;
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit
            || cpu.processing_data.write_count == 2
            || matches!(cpu.processing_data.address, None)
        {
            let is_zero = result == 0;
            let is_carry = operand & 1 != 0;
            cpu.update_processor_status(
                Some(is_carry),
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(false),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Move Block Next
pub fn mvn(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.op_cycles {
        2..=3 => {
            cpu.load_one_arg(memory_controller);
        }
        4 => {
            cpu.processing_data.operand =
                Some(cpu.read_byte(memory_controller, cpu.processing_data.args[1], cpu.x) as u16);
        }
        5 => {
            cpu.data_bank = cpu.processing_data.args[0];
            cpu.write_byte(
                memory_controller,
                cpu.data_bank,
                cpu.y,
                cpu.processing_data.operand.unwrap() as u8,
            );
        }
        6 => {
            cpu.x = cpu.x.wrapping_add(1);
            cpu.y = cpu.y.wrapping_add(1);
            cpu.accumulator = cpu.accumulator.wrapping_sub(1);
        }
        7 => {
            if cpu.accumulator == 0xFFFF {
                cpu.advance_program_counter(3);
            } else {
                cpu.processing_state = LoadingOp;
            }
        }
        _ => (),
    }
}
/// Move Block Previous
pub fn mvp(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.op_cycles {
        2..=3 => {
            cpu.load_one_arg(memory_controller);
        }
        4 => {
            cpu.processing_data.operand =
                Some(cpu.read_byte(memory_controller, cpu.processing_data.args[1], cpu.x) as u16);
        }
        5 => {
            cpu.data_bank = cpu.processing_data.args[0];
            cpu.write_byte(
                memory_controller,
                cpu.data_bank,
                cpu.y,
                cpu.processing_data.operand.unwrap() as u8,
            );
        }
        6 => {
            cpu.x = cpu.x.wrapping_sub(1);
            cpu.y = cpu.y.wrapping_sub(1);
            cpu.accumulator = cpu.accumulator.wrapping_sub(1);
        }
        7 => {
            if cpu.accumulator == 0xFFFF {
                cpu.advance_program_counter(3);
            } else {
                cpu.processing_state = LoadingOp;
            }
        }
        _ => (),
    }
}
/// No Operation
pub fn nop(cpu: &mut CPU) {
    cpu.advance_program_counter(1);
}
/// Or
pub fn ora(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let high_bit_mask;
        let result;
        if is8bit {
            high_bit_mask = 0x80_u16;
            result = (operand as u8 | cpu.accumulator as u8) as u16;
        } else {
            high_bit_mask = 0x8000;
            result = operand | cpu.accumulator;
        }
        cpu.accumulator = result;
        let is_negative = result & high_bit_mask != 0;
        let is_zero = result == 0;
        cpu.update_processor_status(
            None,
            Some(is_zero),
            None,
            None,
            None,
            None,
            None,
            Some(is_negative),
        );
        cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
    }
}
/// Push Effective Address
pub fn pea(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.address {
        None => {
            cpu.cycle_address_loading(&AddressMode::Absolute, memory_controller);
        }
        Some(address) => {
            let address_bytes = split_bytes(address.1);
            let write_complte =
                cpu.cycle_push_stack(memory_controller, &vec![address_bytes.1, address_bytes.0]);
            if write_complte {
                cpu.advance_program_counter(3);
            }
        }
    }
}
/// Push Effective Indirect Address
pub fn pei(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.address {
        None => {
            cpu.cycle_address_loading(&AddressMode::DirectPageIndirect, memory_controller);
        }
        Some(address) => match cpu.processing_data.operand {
            None => {
                cpu.processing_data.operand =
                    Some(cpu.read_byte(memory_controller, address.0, address.1) as u16)
            }
            Some(operand) => {
                cpu.stack_push_byte(memory_controller, operand as u8);
                cpu.advance_program_counter(2);
            }
        },
    }
}
/// Push Effective PC Relative Address
pub fn per(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.operand {
        None => cpu.cycle_operand_loading(&AddressMode::Immediate, memory_controller, false),
        Some(operand) => {
            let op_size = 3;
            let result_addr_bytes = split_bytes(
                cpu.program_counter
                    .wrapping_add(operand)
                    .wrapping_add(op_size),
            );
            let push_complete = cpu.cycle_push_stack(
                memory_controller,
                &vec![result_addr_bytes.1, result_addr_bytes.0],
            );
            if push_complete {
                cpu.advance_program_counter(op_size);
            }
        }
    }
}
/// Push Accumulator
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
            cpu.cycle_push_stack(memory_controller, &vec![bytes.1, bytes.0]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Push Data Bank
pub fn phb(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.write_count < 1 {
        cpu.stack_push_byte(memory_controller, cpu.data_bank);
    }
    if cpu.processing_data.op_cycles == 3 {
        cpu.advance_program_counter(1);
    }
}
/// Push Direct Page
pub fn phd(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let bytes = split_bytes(cpu.direct_page);
    if cpu.processing_data.write_count < 2 {
        cpu.cycle_push_stack(memory_controller, &vec![bytes.1, bytes.0]);
    }

    if cpu.processing_data.op_cycles == 4 {
        cpu.advance_program_counter(1);
    }
}
/// Push Program Bank
pub fn phk(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.write_count < 1 {
        cpu.stack_push_byte(memory_controller, cpu.program_bank);
    }
    if cpu.processing_data.op_cycles == 3 {
        cpu.advance_program_counter(1);
    }
}
/// Push Status
pub fn php(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.write_count < 1 {
        cpu.stack_push_byte(memory_controller, cpu.processor_status);
    }
    if cpu.processing_data.op_cycles == 3 {
        cpu.advance_program_counter(1);
    }
}
/// Push X
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
            cpu.cycle_push_stack(memory_controller, &vec![bytes.1, bytes.0]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Push Y
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
            cpu.cycle_push_stack(memory_controller, &vec![bytes.1, bytes.0]);
        }
        required_cycles += 1;
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull Accumulator
pub fn pla(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    let mut required_cycles = 4_u8;
    if is8bit {
        if cpu.processing_data.read_count == 0 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.accumulator = combine_bytes(0x00, byte);
            let is_negative = Some(check_bits_set(byte, 0b1000_0000));
            let is_zero = Some(byte == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    } else {
        required_cycles += 1;
        if cpu.processing_data.read_count < 2 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.args.push(byte);
        } else if cpu.processing_data.read_count == 2 {
            cpu.accumulator =
                combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
            let is_negative = Some(check_bits_set(cpu.processing_data.args[0], 0b1000_0000));
            let is_zero = Some(cpu.accumulator == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull Data Bank
pub fn plb(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let required_cycles = 4_u8;
    if cpu.processing_data.read_count == 0 {
        let byte = cpu.stack_pop_byte(memory_controller);
        cpu.data_bank = byte;
        let is_negative = Some(check_bits_set(byte, 0b1000_0000));
        let is_zero = Some(byte == 0);
        cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull Direct Page
pub fn pld(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let required_cycles = 5_u8;
    if cpu.processing_data.read_count < 2 {
        let byte = cpu.stack_pop_byte(memory_controller);
        cpu.processing_data.args.push(byte);
    } else if cpu.processing_data.read_count == 2 {
        cpu.direct_page = combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
        let is_negative = Some(check_bits_set(cpu.processing_data.args[0], 0b1000_0000));
        let is_zero = Some(cpu.direct_page == 0);
        cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull Status
pub fn plp(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let required_cycles = 4_u8;
    if cpu.processing_data.read_count == 0 {
        let byte = cpu.stack_pop_byte(memory_controller);
        cpu.set_processor_status(byte);
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull X
pub fn plx(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let mut required_cycles = 4_u8;
    if is8bit {
        if cpu.processing_data.read_count == 0 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.x = combine_bytes(0x00, byte);
            let is_negative = Some(check_bits_set(byte, 0b1000_0000));
            let is_zero = Some(byte == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    } else {
        required_cycles += 1;
        if cpu.processing_data.read_count < 2 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.args.push(byte);
        } else if cpu.processing_data.read_count == 2 {
            cpu.x = combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
            let is_negative = Some(check_bits_set(cpu.processing_data.args[0], 0b1000_0000));
            let is_zero = Some(cpu.x == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Pull Y
pub fn ply(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    let mut required_cycles = 4_u8;
    if is8bit {
        if cpu.processing_data.read_count == 0 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.y = combine_bytes(0x00, byte);
            let is_negative = Some(check_bits_set(byte, 0b1000_0000));
            let is_zero = Some(byte == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    } else {
        required_cycles += 1;
        if cpu.processing_data.read_count < 2 {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.args.push(byte);
        } else if cpu.processing_data.read_count == 2 {
            cpu.y = combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
            let is_negative = Some(check_bits_set(cpu.processing_data.args[0], 0b1000_0000));
            let is_zero = Some(cpu.y == 0);
            cpu.update_processor_status(None, is_zero, None, None, None, None, None, is_negative);
        }
    }
    if cpu.processing_data.op_cycles == required_cycles {
        cpu.advance_program_counter(1);
    }
}
/// Reset Status Bits
pub fn rep(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.args.len() == 0 {
        cpu.load_one_arg(memory_controller);
    } else {
        let arg = cpu.processing_data.args[0];
        let carry = if check_bits_set(arg, CARRY_MASK) {
            Some(false)
        } else {
            None
        };
        let zero = if check_bits_set(arg, ZERO_MASK) {
            Some(false)
        } else {
            None
        };
        let irq_disable = if check_bits_set(arg, IRQ_MASK) {
            Some(false)
        } else {
            None
        };
        let decimal_mode = if check_bits_set(arg, DECIMAL_MASK) {
            Some(false)
        } else {
            None
        };
        let mut index_register_select = if check_bits_set(arg, INDEX_REG_MASK) {
            Some(false)
        } else {
            None
        };
        let mut accumulator_select = if check_bits_set(arg, ACCUMULATOR_MASK) {
            Some(false)
        } else {
            None
        };
        let overflow = if check_bits_set(arg, OVERFLOW_MASK) {
            Some(false)
        } else {
            None
        };
        let negative = if check_bits_set(arg, NEGATIVE_MASK) {
            Some(false)
        } else {
            None
        };

        if cpu.emulation_flag {
            index_register_select = Some(true);
            accumulator_select = Some(true);
        }
        cpu.update_processor_status(
            carry,
            zero,
            irq_disable,
            decimal_mode,
            index_register_select,
            accumulator_select,
            overflow,
            negative,
        );
        cpu.advance_program_counter(2);
    }
}
/// Rotate Left
pub fn rol(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let mut result;
        let high_bit_mask;
        if is8bit {
            high_bit_mask = 0x0080_u16;
            result = ((operand as u8) << 1) as u16;
        } else {
            result = operand << 1;
            high_bit_mask = 0x0080;
        }
        if check_bits_set(cpu.processor_status, CARRY_MASK) {
            result += 1;
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit
            || cpu.processing_data.write_count == 2
            || matches!(cpu.processing_data.address, None)
        {
            let is_carry = operand & high_bit_mask != 0;
            let is_zero = result == 0;
            let is_negative = result & high_bit_mask != 0;
            cpu.update_processor_status(
                Some(is_carry),
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(is_negative),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Rotate Right
pub fn ror(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit);
    }
    if let Some(operand) = cpu.processing_data.operand {
        let mut result;
        let high_bit_mask;
        if is8bit {
            high_bit_mask = 0x0080_u16;
            result = ((operand as u8) >> 1) as u16;
        } else {
            result = operand >> 1;
            high_bit_mask = 0x0080;
        }

        if check_bits_set(cpu.processor_status, CARRY_MASK) {
            result += high_bit_mask;
        }
        match cpu.processing_data.address {
            None => {
                cpu.accumulator = result;
            }
            Some(address) => {
                if is8bit {
                    cpu.write_byte(memory_controller, address.0, address.1, result as u8);
                } else {
                    cpu.cycle_write_word(memory_controller, address.0, address.1, result);
                }
            }
        }
        if is8bit
            || cpu.processing_data.write_count == 2
            || matches!(cpu.processing_data.address, None)
        {
            let is_carry = operand & 1 != 0;
            let is_zero = result == 0;
            let is_negative = result & high_bit_mask != 0;
            cpu.update_processor_status(
                Some(is_carry),
                Some(is_zero),
                None,
                None,
                None,
                None,
                None,
                Some(is_negative),
            );
            cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
        }
    }
}
/// Return from Interrupt
pub fn rti(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.op_cycles {
        2 => (),
        3 => {
            let mut status = cpu.stack_pop_byte(memory_controller);
            if cpu.emulation_flag {
                status = status | ACCUMULATOR_MASK | INDEX_REG_MASK;
            }
            cpu.processor_status = status;
        }
        4..=5 => {
            let byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.args.push(byte);
        }
        6 => {
            cpu.program_counter =
                combine_bytes(cpu.processing_data.args[1], cpu.processing_data.args[0]);
            if cpu.emulation_flag {
                cpu.processing_state = LoadingOp;
            }
        }
        7 => {
            let bank = cpu.stack_pop_byte(memory_controller);
            cpu.program_bank = bank;
            cpu.processing_state = LoadingOp;
        }
        _ => {
            dbg!(cpu.processing_data.op_cycles);
            panic!("Should not be in RTI logic at op cycle count");
        }
    }
}
/// Return from Subroutine Long
pub fn rtl(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.op_cycles {
        2 => {
            cpu.program_bank = cpu.stack_pop_byte(memory_controller);
        }
        3 => {
            let low_byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.operand = Some(low_byte as u16)
        }
        4 => {
            let high_byte = cpu.stack_pop_byte(memory_controller);
            let return_address =
                combine_bytes(high_byte, cpu.processing_data.operand.unwrap() as u8);
            cpu.processing_data.operand = Some(return_address);
        }
        5 => {
            let next_instruction_address = cpu.processing_data.operand.unwrap().wrapping_add(1);
            cpu.processing_data.operand = Some(next_instruction_address);
        }
        6 => {
            cpu.program_counter = cpu.processing_data.operand.unwrap();
            cpu.processing_state = LoadingOp;
        }
        _ => (),
    }
}
/// Return from Subroutine
pub fn rts(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    match cpu.processing_data.op_cycles {
        // Starting at cycle 3 as documentation indicates the stack pointer changes on cycles 3 and 4
        3 => {
            let low_byte = cpu.stack_pop_byte(memory_controller);
            cpu.processing_data.operand = Some(low_byte as u16)
        }
        4 => {
            let high_byte = cpu.stack_pop_byte(memory_controller);
            let return_address =
                combine_bytes(high_byte, cpu.processing_data.operand.unwrap() as u8);
            cpu.processing_data.operand = Some(return_address);
        }
        5 => {
            let next_instruction_address = cpu.processing_data.operand.unwrap().wrapping_add(1);
            cpu.processing_data.operand = Some(next_instruction_address);
        }
        6 => {
            cpu.program_counter = cpu.processing_data.operand.unwrap();
            cpu.processing_state = LoadingOp;
        }
        _ => (),
    }
}
/// Subtract with Carry
pub fn sbc(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    if let None = cpu.processing_data.operand {
        cpu.cycle_operand_loading(address_mode, memory_controller, is8bit)
    }
    if let Some(operand) = cpu.processing_data.operand {
        let carry_subtract_value = if check_bits_set(cpu.processor_status, CARRY_MASK) {
            1_u32
        } else {
            0_u32
        };
        let is_bcd = check_bits_set(cpu.processor_status, DECIMAL_MASK);
        let mut difference;
        let mut is_bcd_overflow = false;
        if is_bcd {
            difference = (get_bdc_value(cpu.accumulator) as u32)
                .wrapping_sub(get_bdc_value(operand) as u32)
                .wrapping_sub(carry_subtract_value);
            is_bcd_overflow = difference > 9999;
            difference = encode_bdc(difference as u16) as u32;
        } else if is8bit {
            difference = ((cpu.accumulator & 0x00FF) as u32)
                .wrapping_sub(operand as u32)
                .wrapping_sub(carry_subtract_value);
        } else {
            difference = (cpu.accumulator as u32)
                .wrapping_sub(operand as u32)
                .wrapping_sub(carry_subtract_value);
        }
        let mut is_overflow = difference & 0x8000_0000 != 0;
        let mut is_negative = if is8bit {
            difference & 0x0080 != 0
        } else {
            difference & 0x8000 != 0
        };
        if is_bcd {
            is_negative = false;
            is_overflow = is_bcd_overflow;
        }
        let is_zero = difference == 0;

        if is8bit {
            difference = difference & 0x0000_00FF;
        }
        cpu.accumulator = difference as u16;
        cpu.update_processor_status(
            Some(!is_negative),
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
/// Set Interrupt Disable Flag
pub fn sec(cpu: &mut CPU) {
    cpu.update_processor_status(Some(true), None, None, None, None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Set Interrupt Disable Flag
pub fn sed(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, None, Some(true), None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Set Interrupt Disable Flag
pub fn sei(cpu: &mut CPU) {
    cpu.update_processor_status(None, None, Some(true), None, None, None, None, None);
    cpu.advance_program_counter(1);
}
/// Set Status Bits
pub fn sep(cpu: &mut CPU, memory_controller: &mut MemoryController) {
    if cpu.processing_data.args.len() == 0 {
        cpu.load_one_arg(memory_controller);
    } else {
        cpu.set_processor_status(cpu.processing_data.args[0]);
        cpu.advance_program_counter(2);
    }
}
/// Store Accumulator
pub fn sta(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    match cpu.processing_data.address {
        None => cpu.cycle_address_loading(address_mode, memory_controller),
        Some(address) => {
            if is8bit {
                cpu.write_byte(
                    memory_controller,
                    address.0,
                    address.1,
                    split_bytes(cpu.x).0,
                );
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, cpu.x);
            }
            if cpu.processing_data.write_count == 2 || is8bit {
                cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
            }
        }
    }
}
/// Stop the Clock
pub fn stp(cpu: &mut CPU) {
    if cpu.processing_data.op_cycles == 3 {
        cpu.clock_stop = true;
        cpu.advance_program_counter(1);
    }
}
/// Store X
pub fn stx(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    match cpu.processing_data.address {
        None => cpu.cycle_address_loading(address_mode, memory_controller),
        Some(address) => {
            if is8bit {
                cpu.write_byte(
                    memory_controller,
                    address.0,
                    address.1,
                    split_bytes(cpu.x).0,
                );
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, cpu.x);
            }
            if cpu.processing_data.write_count == 2 || is8bit {
                cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
            }
        }
    }
}
/// Store Y
pub fn sty(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, INDEX_REG_MASK);
    match cpu.processing_data.address {
        None => cpu.cycle_address_loading(address_mode, memory_controller),
        Some(address) => {
            if is8bit {
                cpu.write_byte(
                    memory_controller,
                    address.0,
                    address.1,
                    split_bytes(cpu.y).0,
                );
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, cpu.y);
            }
            if cpu.processing_data.write_count == 2 || is8bit {
                cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
            }
        }
    }
}
/// Store Zero
pub fn stz(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    match cpu.processing_data.address {
        None => cpu.cycle_address_loading(address_mode, memory_controller),
        Some(address) => {
            if is8bit {
                cpu.write_byte(memory_controller, address.0, address.1, 0x00);
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, 0);
            }
            if cpu.processing_data.write_count == 2 || is8bit {
                if split_bytes(cpu.direct_page).0 != 0 && cpu.processing_data.delay_count == 0 {
                    cpu.processing_data.delay_count += 1;
                    return ();
                }
                cpu.advance_program_counter_with_address_mode(&address_mode, is8bit);
            }
        }
    }
}
/// Register Transfer: Accumulator to X
pub fn tax(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let accu_bytes = split_bytes(cpu.accumulator);
        let x_bytes = split_bytes(cpu.x);
        cpu.x = combine_bytes(x_bytes.1, accu_bytes.0);
        is_negative = accu_bytes.0 & 0x80 != 0;
    } else {
        cpu.x = cpu.accumulator;
        is_negative = cpu.x & 0x8000 != 0;
    }
    let is_zero = cpu.x == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Accumulator to Y
pub fn tay(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let accu_bytes = split_bytes(cpu.accumulator);
        let y_bytes = split_bytes(cpu.y);
        cpu.y = combine_bytes(y_bytes.1, accu_bytes.0);
        is_negative = accu_bytes.0 & 0x80 != 0;
    } else {
        cpu.y = cpu.accumulator;
        is_negative = cpu.y & 0x8000 != 0;
    }
    let is_zero = cpu.y == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Accumulator to Direct Page
pub fn tcd(cpu: &mut CPU) {
    cpu.direct_page = cpu.accumulator;
    let is_negative = cpu.direct_page & 0x8000 != 0;
    let is_zero = cpu.direct_page == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Accumulator to Stack Pointer
pub fn tcs(cpu: &mut CPU) {
    cpu.stack_pointer = cpu.accumulator;
    let is_negative = cpu.stack_pointer & 0x8000 != 0;
    let is_zero = cpu.stack_pointer == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Direct Page to Accumulator
pub fn tdc(cpu: &mut CPU) {
    cpu.accumulator = cpu.direct_page;
    let is_negative = cpu.accumulator & 0x8000 != 0;
    let is_zero = cpu.accumulator == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Test and Reset Bits
pub fn trb(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    match cpu.processing_data.operand {
        None => cpu.cycle_operand_loading(address_mode, memory_controller, is8bit),
        Some(operand) => {
            let accumulator_value = if is8bit {
                cpu.accumulator & 0x00FF
            } else {
                cpu.accumulator
            };
            let result = operand & (!accumulator_value);
            let address = cpu.processing_data.address.unwrap();
            if is8bit {
                cpu.write_byte(memory_controller, address.0, address.1, result as u8);
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, result);
            }
            if cpu.processing_data.delay_count == 0 {
                cpu.processing_data.delay_count += 1;
                return
            }
            if is8bit || cpu.processing_data.write_count == 2 {
                let is_zero = result & accumulator_value == 0;
                cpu.update_processor_status(
                    None,
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
            }
        }
    }
}

/// Test and Set Bits
pub fn tsb(cpu: &mut CPU, memory_controller: &mut MemoryController, address_mode: &AddressMode) {
    let is8bit = check_bits_set(cpu.processor_status, ACCUMULATOR_MASK);
    match cpu.processing_data.operand {
        None => cpu.cycle_operand_loading(address_mode, memory_controller, is8bit),
        Some(operand) => {
            let accumulator_value = if is8bit {
                cpu.accumulator & 0x00FF
            } else {
                cpu.accumulator
            };
            let result = operand | accumulator_value;
            let address = cpu.processing_data.address.unwrap();
            if is8bit {
                cpu.write_byte(memory_controller, address.0, address.1, result as u8);
            } else {
                cpu.cycle_write_word(memory_controller, address.0, address.1, result);
            }
            if cpu.processing_data.delay_count == 0 {
                cpu.processing_data.delay_count += 1;
                return
            }
            if is8bit || cpu.processing_data.write_count == 2 {
                let is_zero = result & accumulator_value == 0;
                cpu.update_processor_status(
                    None,
                    Some(is_zero),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                cpu.advance_program_counter_with_address_mode(address_mode, is8bit);
            }
        }
    }
}
/// Register Transfer: Stack Pointer to Accumulator
pub fn tsc(cpu: &mut CPU) {
    cpu.accumulator = cpu.stack_pointer;
    let is_negative = cpu.accumulator & 0x8000 != 0;
    let is_zero = cpu.accumulator == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Stack Pointer to X
pub fn tsx(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let stack_pointer_bytes = split_bytes(cpu.stack_pointer);
        let x_bytes = split_bytes(cpu.x);
        cpu.x = combine_bytes(x_bytes.1, stack_pointer_bytes.0);
        is_negative = stack_pointer_bytes.0 & 0x80 != 0;
    } else {
        cpu.x = cpu.stack_pointer;
        is_negative = cpu.x & 0x8000 != 0;
    }
    let is_zero = cpu.x == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: X to Accumulator
pub fn txa(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let x_bytes = split_bytes(cpu.x);
        let accu_bytes = split_bytes(cpu.accumulator);
        cpu.accumulator = combine_bytes(accu_bytes.1, x_bytes.0);
        is_negative = x_bytes.0 & 0x80 != 0;
    } else {
        cpu.accumulator = cpu.x;
        is_negative = cpu.accumulator & 0x8000 != 0;
    }
    let is_zero = cpu.accumulator == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: X to Stack Pointer
pub fn txs(cpu: &mut CPU) {
    if cpu.emulation_flag {
        // transfer 8 bits
        let x_bytes = split_bytes(cpu.x);
        let stack_pointer_bytes = split_bytes(cpu.stack_pointer);
        cpu.stack_pointer = combine_bytes(stack_pointer_bytes.1, x_bytes.0);
    } else {
        cpu.stack_pointer = cpu.x;
    }
    cpu.advance_program_counter(1);
}
/// Register Transfer: X to Y
pub fn txy(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let x_bytes = split_bytes(cpu.x);
        let y_bytes = split_bytes(cpu.y);
        cpu.y = combine_bytes(y_bytes.1, x_bytes.0);
        is_negative = x_bytes.0 & 0x80 != 0;
    } else {
        cpu.y = cpu.x;
        is_negative = cpu.y & 0x8000 != 0;
    }
    let is_zero = cpu.y == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Y to Accumulator
pub fn tya(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, ACCUMULATOR_MASK) {
        // transfer 8 bits
        let y_bytes = split_bytes(cpu.y);
        let accum_bytes = split_bytes(cpu.accumulator);
        cpu.accumulator = combine_bytes(accum_bytes.1, y_bytes.0);
        is_negative = y_bytes.0 & 0x80 != 0;
    } else {
        cpu.accumulator = cpu.y;
        is_negative = cpu.accumulator & 0x8000 != 0;
    }
    let is_zero = cpu.accumulator == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Register Transfer: Y to X
pub fn tyx(cpu: &mut CPU) {
    let is_negative;
    if check_bits_set(cpu.processor_status, INDEX_REG_MASK) {
        // transfer 8 bits
        let y_bytes = split_bytes(cpu.y);
        let x_bytes = split_bytes(cpu.x);
        cpu.x = combine_bytes(x_bytes.1, y_bytes.0);
        is_negative = y_bytes.0 & 0x80 != 0;
    } else {
        cpu.x = cpu.y;
        is_negative = cpu.x & 0x8000 != 0;
    }
    let is_zero = cpu.x == 0;
    cpu.update_processor_status(
        None,
        Some(is_zero),
        None,
        None,
        None,
        None,
        None,
        Some(is_negative),
    );
    cpu.advance_program_counter(1);
}
/// Exchange Carry and Emulation Bits
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
            ops::{
                AddressMode::{Absolute, Accumulator},
                Operation,
            },
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
        memory_controller.write(0x00, 0x0000, 0x6D);
        memory_controller.write(0x00, 0x0001, 0x03);
        memory_controller.write(0x00, 0x0002, 0x00);
        memory_controller.write(0x00, 0x0003, 0xFE);
        memory_controller.write(0x00, 0x0004, 0xFF);
        let expected_sum = 0x1234_u16.wrapping_add(0xFFFE).wrapping_add(1);

        // Absolute Mode 16 bit = 5 cycles
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::ADC(Absolute));
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 5);
        assert_eq!(expected_sum, cpu.accumulator);
        assert!(check_bits_set(cpu.processor_status, OVERFLOW_MASK));
    }
    #[test]
    fn asl() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = CARRY_MASK;
        memory_controller.write(0x00, 0x0000, 0x0A);
        cpu.accumulator = 0x89AB;
        let expected_value = 0x89AB << 1;

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::ASL(Accumulator));
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 2);
        assert_eq!(expected_value, cpu.accumulator);
        assert!(check_bits_set(cpu.processor_status, CARRY_MASK));
    }
    #[test]
    fn bcc() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.processor_status = !CARRY_MASK;
        memory_controller.write(0x00, 0x0000, 0x90);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BCC);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 4);
        assert_eq!(cpu.emulation_flag, true);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn bcs() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = CARRY_MASK;
        memory_controller.write(0x00, 0x0000, 0xB0);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BCS);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn beq() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = !ZERO_MASK;
        memory_controller.write(0x00, 0x0000, 0xF0);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BEQ);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 2);
        assert_eq!(cpu.program_counter, 0x0002);
    }
    #[test]
    fn bmi() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = NEGATIVE_MASK;
        memory_controller.write(0x00, 0x0000, 0x30);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BMI);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn bne() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = !ZERO_MASK;
        memory_controller.write(0x00, 0x0000, 0xD0);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BNE);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn bpl() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = !NEGATIVE_MASK;
        memory_controller.write(0x00, 0x0000, 0x10);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BPL);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn bra() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        memory_controller.write(0x00, 0x0000, 0x80);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BRA);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
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
    fn bvc() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = !OVERFLOW_MASK;
        memory_controller.write(0x00, 0x0000, 0x50);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BVC);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(cpu.program_counter, 0xFFFF);
    }
    #[test]
    fn bvs() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = OVERFLOW_MASK;
        memory_controller.write(0x00, 0x0000, 0x70);
        memory_controller.write(0x00, 0x0001, 0xFD);
        memory_controller.write(0x00, 0x0002, 0xFF);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::BVS);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 3);
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
    fn pha() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.accumulator = 0x1234;
        memory_controller.write(0x00, 0x0000, 0x48);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHA);
        let low = cpu.stack_pop_byte(&mut memory_controller);
        let high = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.accumulator, combine_bytes(high, low));
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn phb() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.data_bank = 0x12;
        memory_controller.write(0x00, 0x0000, 0x8B);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHB);
        let byte = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.data_bank, byte);
        assert_eq!(cpu.processing_data.op_cycles, 3);
    }
    #[test]
    fn phd() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.direct_page = 0x1234;
        memory_controller.write(0x00, 0x0000, 0x0B);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHD);
        let low = cpu.stack_pop_byte(&mut memory_controller);
        let high = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.direct_page, combine_bytes(high, low));
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn phk() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.program_bank = 0x12;
        memory_controller.write(0x00, 0x0000, 0x4B);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHK);
        let byte = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.program_bank, byte);
        assert_eq!(cpu.processing_data.op_cycles, 3);
    }
    #[test]
    fn php() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.processor_status = 0x12;
        memory_controller.write(0x00, 0x0000, 0x08);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHP);
        let byte = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.processor_status, byte);
        assert_eq!(cpu.processing_data.op_cycles, 3);
    }
    #[test]
    fn phx() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.x = 0x1234;
        memory_controller.write(0x00, 0x0000, 0xDA);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHX);
        let low = cpu.stack_pop_byte(&mut memory_controller);
        let high = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.x, combine_bytes(high, low));
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn phy() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.y = 0x1234;
        memory_controller.write(0x00, 0x0000, 0x5A);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PHY);
        let low = cpu.stack_pop_byte(&mut memory_controller);
        let high = cpu.stack_pop_byte(&mut memory_controller);
        assert_eq!(cpu.y, combine_bytes(high, low));
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn pla() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.accumulator = 0x0000;
        memory_controller.write(0x00, 0x0000, 0x68);
        cpu.stack_push_byte(&mut memory_controller, 0x12);
        cpu.stack_push_byte(&mut memory_controller, 0x34);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLA);
        assert_eq!(cpu.accumulator, 0x1234);
        assert_eq!(cpu.processing_data.op_cycles, 5);
    }
    #[test]
    fn plb() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.data_bank = 0x00;
        memory_controller.write(0x00, 0x0000, 0xAB);
        cpu.stack_push_byte(&mut memory_controller, 0x12);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLB);
        assert_eq!(cpu.data_bank, 0x12);
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn pld() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.direct_page = 0x0000;
        memory_controller.write(0x00, 0x0000, 0x2B);
        cpu.stack_push_byte(&mut memory_controller, 0x12);
        cpu.stack_push_byte(&mut memory_controller, 0x34);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLD);
        assert_eq!(cpu.direct_page, 0x1234);
        assert_eq!(cpu.processing_data.op_cycles, 5);
    }
    #[test]
    fn plp() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        cpu.processor_status = 0x00;
        memory_controller.write(0x00, 0x0000, 0x28);
        cpu.stack_push_byte(&mut memory_controller, 0x12);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLP);
        assert_eq!(cpu.processor_status, 0x12);
        assert_eq!(cpu.processing_data.op_cycles, 4);
    }
    #[test]
    fn plx() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.x = 0x0000;
        memory_controller.write(0x00, 0x0000, 0xFA);
        cpu.stack_push_byte(&mut memory_controller, 0x12);
        cpu.stack_push_byte(&mut memory_controller, 0x34);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLX);
        assert_eq!(cpu.x, 0x1234);
        assert_eq!(cpu.processing_data.op_cycles, 5);
    }
    #[test]
    fn ply() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.y = 0x0000;
        memory_controller.write(0x00, 0x0000, 0x7A);
        cpu.stack_push_byte(&mut memory_controller, 0x12);
        cpu.stack_push_byte(&mut memory_controller, 0x34);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::PLY);
        assert_eq!(cpu.y, 0x1234);
        assert_eq!(cpu.processing_data.op_cycles, 5);
    }
    #[test]
    fn rep() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.processor_status = 0xFF;
        memory_controller.write(0x00, 0x0000, 0xC2);
        memory_controller.write(0x00, 0x0001, 0b0101_0101);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::REP);
        assert_eq!(0b1011_1010, cpu.processor_status);
        assert_eq!(cpu.processing_data.op_cycles, 3);
    }
    #[test]
    fn rti() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        cpu.emulation_flag = false;
        memory_controller.write(0x00, 0x0000, 0x40);
        cpu.stack_push_byte(&mut memory_controller, 0x12);
        cpu.stack_push_byte(&mut memory_controller, 0x34);
        cpu.stack_push_byte(&mut memory_controller, 0x56);
        cpu.stack_push_byte(&mut memory_controller, 0xAB);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_data.op, Operation::RTI);
        assert_matches!(cpu.processing_state, LoadingOp);
        assert_eq!(cpu.processing_data.op_cycles, 7);
        assert_eq!(cpu.processor_status, 0xAB);
        assert_eq!(cpu.program_bank, 0x12);
        assert_eq!(cpu.program_counter, 0x3456);
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
        assert_matches!(cpu.processing_data.op, Operation::SEI);
        assert_eq!(0b0000_0100, cpu.processor_status);
        assert_eq!(cpu.processing_data.op_cycles, 2);
    }
    #[test]
    fn sep() {
        let mut cpu = CPU::new();
        let mut memory_controller = MemoryController::new();
        cpu.initialize(0x0000);
        memory_controller.write(0x00, 0x0000, 0xE2); // SEP
        memory_controller.write(0x00, 0x0001, 0xFF);
        memory_controller.write(0x00, 0x0002, 0xE2); // SEP
        memory_controller.write(0x00, 0x0003, 0x00);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::SEP);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(0xFF, cpu.processor_status);

        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);
        cpu.process_cycle(&mut memory_controller);

        assert_matches!(cpu.processing_state, LoadingOp);
        assert_matches!(cpu.processing_data.op, Operation::SEP);
        assert_eq!(cpu.processing_data.op_cycles, 3);
        assert_eq!(0b0011_0000, cpu.processor_status);
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
