pub mod emulator {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::bytes::build_string;
    use crate::snes::apu::Apu;
    use crate::snes::cpu::CPU;
    use crate::snes::memory::MemoryController;
    use crate::snes::memory::cartridge::Cartridge;
    pub struct SnesEmulator {
        pub apu: Rc<RefCell<Apu>>,
        pub memory_controller: Rc<RefCell<MemoryController>>,
        pub cpu: CPU,
        pub should_run: bool,
        pub cart_loaded: bool,
    }
    impl SnesEmulator {
        pub fn new() -> SnesEmulator {
            let apu = Rc::new(RefCell::new(Apu::new()));
            let memory_controller = Rc::new(RefCell::new(MemoryController::new(apu.clone())));
            SnesEmulator {
                apu,
                cpu: CPU::new(memory_controller.clone()),
                memory_controller,
                should_run: false,
                cart_loaded: false,
            }
        }
        pub fn load_cartridge(&mut self, cartridge: Cartridge) {
            let mut mem_controller = self.memory_controller.borrow_mut();
            mem_controller.load_cart(cartridge);
            self.cpu.initialize(mem_controller.reset_vector);
            self.cart_loaded = true;
        }
        pub fn activate(&mut self) {
            if self.should_run {
                self.tick();
            }
        }
        pub fn tick(&mut self) {
            self.cpu.tick();
            if self.cpu.clock_stop {
                self.should_run = false
            }
            let mut apu = self.apu.borrow_mut();

            // TODO the apu needs to operate on it's own clock separate from the main cpu clock. THis is a temp hack until proper speeds are implemented
            apu.tick();
        }
        pub fn cycle_cpu(&mut self) {
            self.cpu.process_cycle();
        }
        pub fn execute_cpu_op(&mut self) {
            self.cpu.execute_op();
        }
        pub fn get_cart_title(&self) -> String {
            if self.cart_loaded {
                let title_bytes = self.memory_controller.borrow().read(0, 0xFFC0, 21);
                build_string(&title_bytes)
            } else {
                "Nothing Loaded".to_string()
            }
        }
    }
}
