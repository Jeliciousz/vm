use crate::memory::MemoryController;

const RESET_VECTOR: usize = 0xFFFE;

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
        self.accumulator = 0x00;
        self.memory_controller.reset();
        self.instruction_pointer = self.memory_controller.read_16(RESET_VECTOR);
    }
}