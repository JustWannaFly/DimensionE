pub mod emulator {
    use crate::snes::cpu::CPU;
    use crate::snes::memory::cartridge::Cartridge;
use crate::snes::memory::{MemoryController};
    pub struct SnesEmulator {
        pub memory_controller: MemoryController,
        pub cpu: CPU,
        pub tick: u8,
    }
    impl SnesEmulator {
        pub fn new() -> SnesEmulator {
            SnesEmulator { 
                memory_controller: MemoryController::new(),
                cpu: CPU::new(),
                tick: 0,
            }
        }
        pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
            self.memory_controller.load_cart(cartridge);
            self.cpu.initialize(self.memory_controller.reset_vector);
        }
        pub fn tick(&mut self) {
            self.cpu.tick(self.tick, &mut self.memory_controller);
            self.tick = self.tick.wrapping_add(1);
        }
    }
}
