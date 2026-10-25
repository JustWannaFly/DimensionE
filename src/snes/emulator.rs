pub mod emulator {
    use crate::bytes::build_string;
    use crate::snes::cpu::CPU;
    use crate::snes::memory::MemoryController;
    use crate::snes::memory::cartridge::Cartridge;
    pub struct SnesEmulator {
        pub memory_controller: MemoryController,
        pub cpu: CPU,
        pub should_run: bool,
        pub cart_loaded: bool,
    }
    impl SnesEmulator {
        pub fn new() -> SnesEmulator {
            SnesEmulator {
                memory_controller: MemoryController::new(),
                cpu: CPU::new(),
                should_run: false,
                cart_loaded: false,
            }
        }
        pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
            self.memory_controller.load_cart(cartridge);
            self.cpu.initialize(self.memory_controller.reset_vector);
            self.cart_loaded = true;
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
        pub fn get_cart_title(&self) -> String {
            if self.cart_loaded {
                let title_bytes = self.memory_controller.read(0, 0xFFC0, 21);
                build_string(&title_bytes)
            } else {
                "Nothing Loaded".to_string()
            }
        }
    }
}
