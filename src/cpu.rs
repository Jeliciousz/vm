use crate::memory::MemoryController;

pub const ADDRESS_BUS_WIDTH: u32 = 16;
pub const RESET_VECTOR: usize = 0xFFFC;
pub const NMI_VECTOR: usize = 0xFFFE;

pub struct CPU {
    pub memory_controller: MemoryController,
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub index: u16,
    pub status: u8,     // ionzc
    pub accumulator: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory_controller: MemoryController::new(),
            program_counter: 0x0000,
            stack_pointer: 0x0000,
            index: 0x0000,
            status: 0b00000000,
            accumulator: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
        }
    }

    pub fn reset(&mut self) {
        self.memory_controller.reset();
        self.program_counter = self.memory_controller.read_16(RESET_VECTOR);
        self.stack_pointer = 0x0000;
        self.index = 0x0000;
        self.status = 0b00000000;
        self.accumulator = 0x00;
        self.b = 0x00;
        self.c = 0x00;
        self.d = 0x00;
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