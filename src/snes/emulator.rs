pub mod emulator {
    use crate::snes::cpu::CPU;
    use crate::snes::memory::cartridge::Cartridge;
use crate::snes::memory::{MemoryController};
    pub struct SnesEmulator {
        pub memory_controller: MemoryController,
        pub cpu: CPU,
        pub should_run: bool,
    }
    impl SnesEmulator {
        pub fn new() -> SnesEmulator {
            SnesEmulator { 
                memory_controller: MemoryController::new(),
                cpu: CPU::new(),
                should_run: false,
            }
        }
        pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
            self.memory_controller.load_cart(cartridge);
            self.cpu.initialize(self.memory_controller.reset_vector);
        }
        pub fn activate(&mut self) {
            if self.should_run {
                self.tick();
            }
        }
        pub fn tick(&mut self) {
            self.cpu.tick(&mut self.memory_controller);
        }
        pub fn execute_cpu_op(&mut self) {
            self.cpu.execute_op(&mut self.memory_controller);
        }
    }
}
