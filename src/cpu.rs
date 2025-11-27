use crate::memory::MemoryController;

const RESET_VECTOR: u16 = 0xFFFE;

pub struct CPU<'cpu> {
    pub instruction_pointer: u16,
    pub accumulator: u8,
    pub memory_controller: MemoryController<'cpu>,
}

impl<'cpu> CPU<'cpu> {
    pub fn new() -> Self {
        Self {
            instruction_pointer: 0x0000,
            accumulator: 0x00,
            memory_controller: MemoryController::new(),
        }
    }

    pub fn reset(&mut self) {
        self.instruction_pointer = RESET_VECTOR;
        self.accumulator = 0x00;
        self.memory_controller.reset();
    }
}