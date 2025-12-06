use crate::memory::MemoryController;

pub const DATA_BUS_WIDTH: u32 = 8;
pub const ADDRESS_BUS_WIDTH: u32 = 16;
pub const RESET_VECTOR: usize = 0xFFFE;

pub struct CPU {
    pub memory_controller: MemoryController,
    pub program_counter: u16,
    pub accumulator: u8,
    pub status: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory_controller: MemoryController::new(),
            program_counter: 0x0000,
            accumulator: 0x00,
            status: 0b00000000,
        }
    }

    pub fn reset(&mut self) {
        self.memory_controller.reset();
        self.program_counter = self.memory_controller.read_16(RESET_VECTOR);
        self.accumulator = 0x00;
        self.status = 0b00000000;
    }

    pub fn step(&mut self) {
        match self.fetch() {
            0x00 => { // LDA IMM
                self.accumulator = self.fetch();
            },
            0xF0 => { // HLT
                self.program_counter -= 1;
            },
            0xF1 => { // RST
                self.reset();
            },
            _ => () // NOP
        }
    }

    fn fetch(&mut self) -> u8 {
        let fetched_value = self.memory_controller.read_8(self.program_counter as usize);
        self.program_counter += 1;
        fetched_value
    }
}