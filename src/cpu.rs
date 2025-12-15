use crate::memory::MemoryController;

pub const ADDRESS_BUS_WIDTH: u32 = 16;
pub const RESET_VECTOR: usize = 0xFFFC;
pub const NMI_VECTOR: usize = 0xFFFE;

pub struct CPU {
    pub memory_controller: MemoryController,
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub index_x: u16,
    pub index_y: u16,
    pub status: u16,     // ionzc
    pub accumulator: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory_controller: MemoryController::new(),
            program_counter: 0x0000,
            stack_pointer: 0x0000,
            index_x: 0x0000,
            index_y: 0x0000,
            status: 0b0000_0000_0000_0000,
            accumulator: 0x0000,
            b: 0x0000,
            c: 0x0000,
            d: 0x0000,
        }
    }

    pub fn reset(&mut self) {
        self.memory_controller.reset();
        self.program_counter = self.memory_controller.read_16(RESET_VECTOR);
        self.stack_pointer = 0x0000;
        self.index_x = 0x0000;
        self.index_y = 0x0000;
        self.status = 0b0000_0000_0000_0000;
        self.accumulator = 0x0000;
        self.b = 0x0000;
        self.c = 0x0000;
        self.d = 0x0000;
    }

    pub fn step(&mut self) {
        let fetched_instruction = self.fetch16();

        let opcode = (fetched_instruction & 0x007F) as u8;
        let byte_mode = (fetched_instruction & 0x0080) >> 7 != 0;
        let destination = ((fetched_instruction & 0xF000) >> 12) as u8;
        let source = ((fetched_instruction & 0x0F00) >> 8) as u8;

        match opcode {
            0x00 => { // MOV
                self.execute_mov(byte_mode, destination, source);
            },
            0x70 => { // HLT
                self.program_counter -= 2;
            },
            0x71 => { // RST
                self.reset();
            },
            _ => () // NOP
        }
    }

    fn fetch16(&mut self) -> u16 {
        let fetched_value = self.memory_controller.read_16(self.program_counter as usize);
        self.program_counter += 2;
        fetched_value
    }

    fn fetch8(&mut self) -> u8 {
        let fetched_value = self.memory_controller.read_8(self.program_counter as usize);
        self.program_counter += 1;
        fetched_value
    }

    fn fetch_immediate16(&mut self, byte_mode: bool) -> u16 {
        if byte_mode {
            self.fetch8() as u16
        } else {
            self.fetch16()
        }
    }

    fn fetch_value16(&self, byte_mode: bool, address: usize) -> u16 {
        if byte_mode {
            self.memory_controller.read_8(address) as u16
        } else {
            self.memory_controller.read_16(address)
        }
    }

    fn fetch_address(&mut self) -> usize {
        self.fetch16() as usize
    }

    fn fetch_indexed_address(&mut self) -> usize {
        (self.fetch16() + self.index_x) as usize
    }

    fn fetch_indirect_address(&mut self) -> usize {
        let indirect_address = self.fetch16() as usize;
        self.memory_controller.read_16(indirect_address) as usize
    }

    fn fetch_indirect_indexed_address(&mut self) -> usize {
        let indirect_address = self.fetch16() as usize;
        (self.memory_controller.read_16(indirect_address) + self.index_x) as usize
    }

    fn fetch_indexed_indirect_address(&mut self) -> usize {
        let indirect_address = (self.fetch16() + self.index_x) as usize;
        self.memory_controller.read_16(indirect_address) as usize
    }

    fn get_pointer_indexed_address(&mut self) -> usize {
        (self.index_y + self.index_x) as usize
    }

    fn get_pointer_indirect_address(&mut self) -> usize {
        let indirect_address = self.index_y as usize;
        self.memory_controller.read_16(indirect_address) as usize
    }

    fn get_pointer_indirect_indexed_address(&mut self) -> usize {
        let indirect_address = self.index_y as usize;
        (self.memory_controller.read_16(indirect_address) + self.index_x) as usize
    }

    fn get_pointer_indexed_indirect_address(&mut self) -> usize {
        let indirect_address = (self.index_y + self.index_x) as usize;
        self.memory_controller.read_16(indirect_address) as usize
    }

    fn execute_mov(&mut self, byte_mode: bool, destination: u8, source: u8) {
        let source_value: u16 = match source {
            0x0 => self.fetch_immediate16(byte_mode),
            0x1 => self.accumulator,
            0x2 => self.b,
            0x3 => self.c,
            0x4 => self.d,
            0x5 => self.index_x,
            0x6 => self.index_y,
            0x7 => { // ADDR
                let source_address = self.fetch_address();

                self.fetch_value16(byte_mode, source_address)
            },
            0x8 => { // ADDR,IDX
                let source_address = self.fetch_indexed_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            0x9 => { // (ADDR)
                let source_address = self.fetch_indirect_address();

                self.fetch_value16(byte_mode, source_address)
            },
            0xA => { // (ADDR),IDX
                let source_address = self.fetch_indirect_indexed_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            0xB => { // (ADDR,IDX)
                let source_address = self.fetch_indexed_indirect_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            0xC => { // IDY,IDX
                let source_address = self.get_pointer_indexed_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            0xD => { // (IDY)
                let source_address = self.get_pointer_indirect_address();

                self.fetch_value16(byte_mode, source_address)
            },
            0xE => { // (IDY),IDX
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            0xF => { // (IDY,IDX)
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.fetch_value16(byte_mode, source_address)
            },
            _ => 0
        };

        match destination {
            0x0 => (), // IMM (NOP)
            0x1 => { // A
                if byte_mode {
                    self.accumulator = self.accumulator & 0xFF00 | source_value;
                } else {
                    self.accumulator = source_value;
                }
            },
            0x2 => { // B
                if byte_mode {
                    self.b = self.b & 0xFF00 | source_value;
                } else {
                    self.b = source_value;
                }
            },
            0x3 => { // C
                if byte_mode {
                    self.c = self.c & 0xFF00 | source_value;
                } else {
                    self.c = source_value;
                }
            },
            0x4 => { // D
                if byte_mode {
                    self.d = self.d & 0xFF00 | source_value;
                } else {
                    self.d = source_value;
                }
            },
            0x5 => { // IDX
                if byte_mode {
                    self.index_x = self.index_x & 0xFF00 | source_value;
                } else {
                    self.index_x = source_value;
                }
            },
            0x6 => { // IDY
                if byte_mode {
                    self.index_y = self.index_y & 0xFF00 | source_value;
                } else {
                    self.index_y = source_value;
                }
            },
            0x7 => { // ADDR
                let destination_address = self.fetch_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0x8 => { // ADDR,IDX
                let destination_address = self.fetch_indexed_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0x9 => { // (ADDR)
                let destination_address = self.fetch_indirect_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xA => { // (ADDR),IDX
                let destination_address = self.fetch_indirect_indexed_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xB => { // (ADDR,IDX)
                let destination_address = self.fetch_indexed_indirect_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xC => { // *IDY,IDX
                let destination_address = self.get_pointer_indexed_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xD => { // (*IDY)
                let destination_address = self.get_pointer_indirect_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xE => { // (*IDY),IDX
                let destination_address = self.get_pointer_indirect_indexed_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            0xF => { // (*IDY,IDX)
                let destination_address = self.get_pointer_indexed_indirect_address();

                if byte_mode {
                    self.memory_controller.write_8(destination_address, source_value as u8);
                } else {
                    self.memory_controller.write_16(destination_address, source_value);
                }
            },
            _ => ()
        }
    }
}