use std::{cell::RefCell, rc::Rc};

use crate::snes::apu::{AramController, spc700::ProcessingState::{LoadingOp, Processing}};


const BASE_CPU_SPEED: u8 = 1; // set real value here. This is dummy placeholder

#[derive(Debug)]
enum ProcessingState {
    LoadingOp,
    Processing,
}
pub struct Spc700 {
    accumulator: u8,
    x: u8,
    y: u8,
    stack_pointer: u8,
    program_counter: u16,
    psw: u8,

    aram_controller: Rc<RefCell<AramController>>,
    ticks_since_cycle: u8,
    processing_state: ProcessingState,
}

impl Spc700 {
    pub fn new(aram_controller: Rc<RefCell<AramController>>) -> Spc700 {
        Spc700 {
            accumulator: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: 0,
            psw: 0,

            aram_controller,
            ticks_since_cycle: 0,
            processing_state: LoadingOp,
        }
    }
    pub fn tick(&mut self) {
        self.ticks_since_cycle += 1;
        if self.ticks_since_cycle >= BASE_CPU_SPEED {
            // println!("");
            self.ticks_since_cycle = 0;
            self.process_cycle();
        }
        // print!("|");
    }
    pub fn process_cycle(&mut self) {
        match &self.processing_state {
            LoadingOp => {
                // println!(
                // "Cycles taken for op: [{}] :: Total Ops: [{}]",
                // self.processing_data.op_cycles, self.ops_executed
                // );
                let op_code = self.aram_controller.borrow().read_byte(self.program_counter);
                // self.processing_data = ProcessingDetails {
                //     op: lookup_opcode(op_code),
                //     args: vec![],
                //     operand: None,
                //     address: None,
                //     op_cycles: 1,
                //     write_count: 0,
                //     read_count: 0,
                //     delay_count: 0,
                // };
                println!(
                    "Next instruction: [{:X}:{:X}]:[{:X}:{:?}]",
                    self.program_bank, self.program_counter, op_code, self.processing_data.op
                );
                self.processing_state = Processing;
            }
            Processing => {
                self.processing_data.op_cycles += 1;
                self.cycle_execute_op();
            }
        }
    }
}
