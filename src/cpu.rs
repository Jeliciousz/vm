use crate::memory::MemoryController;

pub const DATA_BUS_WIDTH: u32 = 8;
pub const ADDRESS_BUS_WIDTH: u32 = 16;
pub const RESET_VECTOR: usize = 0xFFFE;

pub struct CPU {
    pub memory_controller: MemoryController,
    pub instruction_pointer: u16,
    pub accumulator: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory_controller: MemoryController::new(),
            instruction_pointer: 0x0000,
            accumulator: 0x00,
        }
    }

    pub fn reset(&mut self) {
        self.memory_controller.reset();
        self.instruction_pointer = self.memory_controller.read_16(RESET_VECTOR);
        self.accumulator = 0x00;
    }
}