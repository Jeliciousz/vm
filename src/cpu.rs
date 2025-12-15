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
    pub status: u8, // szpc_oi
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
            status: 0b0000_0000,
            accumulator: 0x0000,
            b: 0x0000,
            c: 0x0000,
            d: 0x0000,
        }
    }

    pub fn reset(&mut self) {
        self.memory_controller.reset();
        self.program_counter = self.memory_controller.read16(RESET_VECTOR);
        self.stack_pointer = 0x0000;
        self.index_x = 0x0000;
        self.index_y = 0x0000;
        self.status = 0b0000_0000;
        self.accumulator = 0x0000;
        self.b = 0x0000;
        self.c = 0x0000;
        self.d = 0x0000;
    }

    pub fn step(&mut self) {
        let fetched_instruction = self.fetch16();

        let operation = (fetched_instruction & 0x007F) as u8;
        let byte_mode = (fetched_instruction & 0x0080) >> 7 != 0;
        let destination = ((fetched_instruction & 0xF000) >> 12) as u8;
        let source = ((fetched_instruction & 0x0F00) >> 8) as u8;

        match operation {
            0x00 => { // MOV
                if byte_mode {
                    self.execute_mov8(destination, source);
                } else {
                    self.execute_mov16(destination, source);
                }
            },
            0x01 => { // ADC
                if byte_mode {
                    self.execute_adc8(destination, source);
                } else {
                    self.execute_adc16(destination, source);
                }
            },
            0x70 => { // HLT
                self.program_counter -= 2;
            },
            0x71 => { // RST
                self.reset();
            },
            _ => () // NOP (not more than 128 operations)
        }
    }

    fn fetch16(&mut self) -> u16 {
        let fetched_value = self.memory_controller.read16(self.program_counter as usize);
        self.program_counter += 2;
        fetched_value
    }

    fn fetch8(&mut self) -> u8 {
        let fetched_value = self.memory_controller.read8(self.program_counter as usize);
        self.program_counter += 1;
        fetched_value
    }

    fn fetch_address(&mut self) -> usize {
        self.fetch16() as usize
    }

    fn fetch_indexed_address(&mut self) -> usize {
        (self.fetch16() + self.index_x) as usize
    }

    fn fetch_indirect_address(&mut self) -> usize {
        let indirect_address = self.fetch16() as usize;
        self.memory_controller.read16(indirect_address) as usize
    }

    fn fetch_indirect_indexed_address(&mut self) -> usize {
        let indirect_address = self.fetch16() as usize;
        (self.memory_controller.read16(indirect_address) + self.index_x) as usize
    }

    fn fetch_indexed_indirect_address(&mut self) -> usize {
        let indirect_address = (self.fetch16() + self.index_x) as usize;
        self.memory_controller.read16(indirect_address) as usize
    }

    fn get_pointer_indexed_address(&mut self) -> usize {
        (self.index_y + self.index_x) as usize
    }

    fn get_pointer_indirect_address(&mut self) -> usize {
        let indirect_address = self.index_y as usize;
        self.memory_controller.read16(indirect_address) as usize
    }

    fn get_pointer_indirect_indexed_address(&mut self) -> usize {
        let indirect_address = self.index_y as usize;
        (self.memory_controller.read16(indirect_address) + self.index_x) as usize
    }

    fn get_pointer_indexed_indirect_address(&mut self) -> usize {
        let indirect_address = (self.index_y + self.index_x) as usize;
        self.memory_controller.read16(indirect_address) as usize
    }

    fn get_sign_flag(&self) -> bool {
        self.status & 0x80 != 0
    }

    fn get_zero_flag(&self) -> bool {
        self.status & 0x40 != 0
    }

    fn get_parity_flag(&self) -> bool {
        self.status & 0x20 != 0
    }

    fn get_carry_flag(&self) -> bool {
        self.status & 0x10 != 0
    }

    fn get_overflow_flag(&self) -> bool {
        self.status & 0x08 != 0
    }

    fn get_interrupt_enable_flag(&self) -> bool {
        self.status & 0x04 != 0
    }

    fn set_sign_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x80;
        } else {
            self.status &= 0x7F
        }
    }

    fn set_zero_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x40;
        } else {
            self.status &= 0xBF;
        }
    }

    fn set_parity_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x20;
        } else {
            self.status &= 0xDF;
        }
    }

    fn set_carry_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x10;
        } else {
            self.status &= 0xEF;
        }
    }

    fn set_overflow_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x08;
        } else {
            self.status &= 0xF7;
        }
    }

    fn set_interrupt_enable_flag(&mut self, flag: bool) {
        if flag {
            self.status |= 0x04;
        } else {
            self.status &= 0xFB;
        }
    }

    fn set_flags_from_value16(&mut self, value: u16) {
        self.set_sign_flag(value & 0x80 != 0);
        self.set_zero_flag(value == 0);
        self.set_parity_flag(value.count_ones() % 2 == 0);
    }

    fn set_flags_from_value8(&mut self, value: u8) {
        self.set_sign_flag(value & 0x80 != 0);
        self.set_zero_flag(value == 0);
        self.set_parity_flag(value.count_ones() % 2 == 0);
    }

    fn set_accumulator8(&mut self, value: u8) {
        self.accumulator = self.accumulator & 0xFF00 | value as u16;
    }

    fn set_b8(&mut self, value: u8) {
        self.b = self.b & 0xFF00 | value as u16;
    }

    fn set_c8(&mut self, value: u8) {
        self.c = self.c & 0xFF00 | value as u16;
    }

    fn set_d8(&mut self, value: u8) {
        self.d = self.d & 0xFF00 | value as u16;
    }

    fn set_index_x8(&mut self, value: u8) {
        self.index_x = self.index_x & 0xFF00 | value as u16;
    }

    fn set_index_y8(&mut self, value: u8) {
        self.index_y = self.index_y & 0xFF00 | value as u16;
    }

    fn add_with_carry16(&mut self, lhs: u16, rhs: u16, carry: bool) -> u16 {
        let (result, carry_out) = lhs.carrying_add(rhs, carry);

        self.set_flags_from_value16(result);
        self.set_carry_flag(carry_out);
        self.set_overflow_flag(lhs & 0x8000 == rhs & 0x8000 && lhs & 0x8000 != result & 0x8000);

        result
    }

    fn add_with_carry8(&mut self, lhs: u8, rhs: u8, carry: bool) -> u8 {
        let (result, carry_out) = lhs.carrying_add(rhs, carry);

        self.set_flags_from_value8(result);
        self.set_carry_flag(carry_out);
        self.set_overflow_flag(lhs & 0x80 == rhs & 0x80 && lhs & 0x80 != result & 0x80);

        result
    }

    fn execute_mov16(&mut self, destination: u8, source: u8) {
        let source_value = match source {
            0x0 => self.fetch16(),
            0x1 => self.accumulator,
            0x2 => self.b,
            0x3 => self.c,
            0x4 => self.d,
            0x5 => self.index_x,
            0x6 => self.index_y,
            0x7 => { // ADDR
                let source_address = self.fetch_address();

                self.memory_controller.read16(source_address)
            },
            0x8 => { // ADDR,IDX
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0x9 => { // (ADDR)
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read16(source_address)
            },
            0xA => { // (ADDR),IDX
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xB => { // (ADDR,IDX)
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            0xC => { // IDY,IDX
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xD => { // (IDY)
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read16(source_address)
            },
            0xE => { // (IDY),IDX
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xF => { // (IDY,IDX)
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            _ => 0
        };

        self.set_flags_from_value16(source_value);

        match destination {
            0x0 => (), // IMM (NOP)
            0x1 => { // A
                self.accumulator = source_value;
            },
            0x2 => { // B
                self.b = source_value;
            },
            0x3 => { // C
                self.c = source_value;
            },
            0x4 => { // D
                self.d = source_value;
            },
            0x5 => { // IDX
                self.index_x = source_value;
            },
            0x6 => { // IDY
                self.index_y = source_value;
            },
            0x7 => { // ADDR
                let destination_address = self.fetch_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0x8 => { // ADDR,IDX
                let destination_address = self.fetch_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0x9 => { // (ADDR)
                let destination_address = self.fetch_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xA => { // (ADDR),IDX
                let destination_address = self.fetch_indirect_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xB => { // (ADDR,IDX)
                let destination_address = self.fetch_indexed_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xC => { // *IDY,IDX
                let destination_address = self.get_pointer_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xD => { // (*IDY)
                let destination_address = self.get_pointer_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xE => { // (*IDY),IDX
                let destination_address = self.get_pointer_indirect_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            0xF => { // (*IDY,IDX)
                let destination_address = self.get_pointer_indexed_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            _ => ()
        }
    }

    fn execute_mov8(&mut self, destination: u8, source: u8) {
        let source_value = match source {
            0x0 => self.fetch8(),
            0x1 => self.accumulator as u8,
            0x2 => self.b as u8,
            0x3 => self.c as u8,
            0x4 => self.d as u8,
            0x5 => self.index_x as u8,
            0x6 => self.index_y as u8,
            0x7 => { // ADDR
                let source_address = self.fetch_address();

                self.memory_controller.read8(source_address)
            },
            0x8 => { // ADDR,IDX
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0x9 => { // (ADDR)
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read8(source_address)
            },
            0xA => { // (ADDR),IDX
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xB => { // (ADDR,IDX)
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            0xC => { // IDY,IDX
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xD => { // (IDY)
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read8(source_address)
            },
            0xE => { // (IDY),IDX
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xF => { // (IDY,IDX)
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            _ => 0
        };

        self.set_flags_from_value8(source_value);

        match destination {
            0x0 => (), // IMM (NOP)
            0x1 => { // A
                self.set_accumulator8(source_value);
            },
            0x2 => { // B
                self.set_b8(source_value);
            },
            0x3 => { // C
                self.set_c8(source_value);
            },
            0x4 => { // D
                self.set_d8(source_value);
            },
            0x5 => { // IDX
                self.set_index_x8(source_value);
            },
            0x6 => { // IDY
                self.set_index_y8(source_value);
            },
            0x7 => { // ADDR
                let destination_address = self.fetch_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0x8 => { // ADDR,IDX
                let destination_address = self.fetch_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0x9 => { // (ADDR)
                let destination_address = self.fetch_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xA => { // (ADDR),IDX
                let destination_address = self.fetch_indirect_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xB => { // (ADDR,IDX)
                let destination_address = self.fetch_indexed_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xC => { // *IDY,IDX
                let destination_address = self.get_pointer_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xD => { // (*IDY)
                let destination_address = self.get_pointer_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xE => { // (*IDY),IDX
                let destination_address = self.get_pointer_indirect_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            0xF => { // (*IDY,IDX)
                let destination_address = self.get_pointer_indexed_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            _ => ()
        }
    }

    fn execute_adc16(&mut self, destination: u8, source: u8) {
        let source_value = match source {
            0x0 => self.fetch16(),
            0x1 => self.accumulator,
            0x2 => self.b,
            0x3 => self.c,
            0x4 => self.d,
            0x5 => self.index_x,
            0x6 => self.index_y,
            0x7 => { // ADDR
                let source_address = self.fetch_address();

                self.memory_controller.read16(source_address)
            },
            0x8 => { // ADDR,IDX
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0x9 => { // (ADDR)
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read16(source_address)
            },
            0xA => { // (ADDR),IDX
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xB => { // (ADDR,IDX)
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            0xC => { // IDY,IDX
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xD => { // (IDY)
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read16(source_address)
            },
            0xE => { // (IDY),IDX
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            0xF => { // (IDY,IDX)
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            _ => 0
        };

        match destination {
            0x0 => (), // IMM (NOP)
            0x1 => { // A
                self.accumulator = self.add_with_carry16(self.accumulator, source_value, self.get_carry_flag());
            },
            0x2 => { // B
                self.b = self.add_with_carry16(self.b, source_value, self.get_carry_flag());
            },
            0x3 => { // C
                self.c = self.add_with_carry16(self.c, source_value, self.get_carry_flag());
            },
            0x4 => { // D
                self.d = self.add_with_carry16(self.d, source_value, self.get_carry_flag());
            },
            0x5 => { // IDX
                self.index_x = self.add_with_carry16(self.index_x, source_value, self.get_carry_flag());
            },
            0x6 => { // IDY
                self.index_y = self.add_with_carry16(self.index_y, source_value, self.get_carry_flag());
            },
            0x7 => { // ADDR
                let destination_address = self.fetch_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0x8 => { // ADDR,IDX
                let destination_address = self.fetch_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0x9 => { // (ADDR)
                let destination_address = self.fetch_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xA => { // (ADDR),IDX
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xB => { // (ADDR,IDX)
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xC => { // *IDY,IDX
                let destination_address = self.get_pointer_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xD => { // (*IDY)
                let destination_address = self.get_pointer_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xE => { // (*IDY),IDX
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            0xF => { // (*IDY,IDX)
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            _ => ()
        }
    }

    fn execute_adc8(&mut self, destination: u8, source: u8) {
        let source_value = match source {
            0x0 => self.fetch8(),
            0x1 => self.accumulator as u8,
            0x2 => self.b as u8,
            0x3 => self.c as u8,
            0x4 => self.d as u8,
            0x5 => self.index_x as u8,
            0x6 => self.index_y as u8,
            0x7 => { // ADDR
                let source_address = self.fetch_address();

                self.memory_controller.read8(source_address)
            },
            0x8 => { // ADDR,IDX
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0x9 => { // (ADDR)
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read8(source_address)
            },
            0xA => { // (ADDR),IDX
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xB => { // (ADDR,IDX)
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            0xC => { // IDY,IDX
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xD => { // (IDY)
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read8(source_address)
            },
            0xE => { // (IDY),IDX
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            0xF => { // (IDY,IDX)
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            _ => 0
        };

        match destination {
            0x0 => (), // IMM (NOP)
            0x1 => { // A
                let result = self.add_with_carry8(self.accumulator as u8, source_value, self.get_carry_flag());
                self.set_accumulator8(result);
            },
            0x2 => { // B
                let result = self.add_with_carry8(self.b as u8, source_value, self.get_carry_flag());
                self.set_b8(result);
            },
            0x3 => { // C
                let result = self.add_with_carry8(self.c as u8, source_value, self.get_carry_flag());
                self.set_c8(result);
            },
            0x4 => { // D
                let result = self.add_with_carry8(self.d as u8, source_value, self.get_carry_flag());
                self.set_d8(result);
            },
            0x5 => { // IDX
                let result = self.add_with_carry8(self.index_x as u8, source_value, self.get_carry_flag());
                self.set_index_x8(result);
            },
            0x6 => { // IDY
                let result = self.add_with_carry8(self.index_y as u8, source_value, self.get_carry_flag());
                self.set_index_y8(result);
            },
            0x7 => { // ADDR
                let destination_address = self.fetch_address();
                
                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0x8 => { // ADDR,IDX
                let destination_address = self.fetch_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0x9 => { // (ADDR)
                let destination_address = self.fetch_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xA => { // (ADDR),IDX
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xB => { // (ADDR,IDX)
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xC => { // *IDY,IDX
                let destination_address = self.get_pointer_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xD => { // (*IDY)
                let destination_address = self.get_pointer_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xE => { // (*IDY),IDX
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            0xF => { // (*IDY,IDX)
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            _ => ()
        }
    }
}