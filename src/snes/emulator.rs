pub mod emulator {
    use crate::snes::cpu::CPU;
    use crate::snes::memory::MemoryController;
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
    }
}
