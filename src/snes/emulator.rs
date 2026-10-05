pub mod emulator {
    use crate::snes::cpu::CPU;
    use crate::snes::memory::cartridge::Cartridge;
use crate::snes::memory::{MemoryController};
    pub struct SnesEmulator {
        pub memory_controller: MemoryController,
        pub cpu: CPU,
    }
    impl SnesEmulator {
        pub fn new() -> SnesEmulator {
            SnesEmulator { 
                memory_controller: MemoryController::new(),
                cpu: CPU::new()
            }
        }
        pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
            self.memory_controller.load_cart(cartridge);
            self.cpu.initialize(self.memory_controller.reset_vector);
        }
        pub fn tick(&mut self) {
            self.cpu.execute_instruction(&mut self.memory_controller);
        }
    }
}
