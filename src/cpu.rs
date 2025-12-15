use core::panic;

use crate::memory::MemoryController;

pub const ADDRESS_BUS_WIDTH: u32 = 16;
pub const RESET_VECTOR: usize = 0xFFFE;
pub const NMI_VECTOR: usize = 0xFFFC;
pub const IRQ_VECTOR: usize = 0xFFFA;

enum Operation {
    Mov,
    Adc,
    Sbc,
    Stp,
    Rst,
    Nop,
}

impl Operation {
    fn get_operation_from_instruction(instruction: u16) -> Self {
        let operation = instruction & 0x003F;

        match operation {
            0x00 => Self::Mov,
            0x01 => Self::Adc,
            0x02 => Self::Sbc,
            0x30 => Self::Stp,
            0x31 => Self::Rst,
            _ => Self::Nop,
        }
    }
}

enum Location {
    Immediate,
    A,
    B,
    C,
    D,
    Idx,
    Idy,
    Address,
    IndexedAddress,
    IndirectAddress,
    IndirectIndexedAddress,
    IndexedIndirectAddress,
    IndexedPointer,
    IndirectPointer,
    IndirectIndexedPointer,
    IndexedIndirectPointer,
}

impl Location {
    fn get_destination_from_instruction(instruction: u16) -> Self {
        let destination = (instruction & 0xF000) >> 12;

        match destination {
            0x0 => Self::Immediate,
            0x1 => Self::A,
            0x2 => Self::B,
            0x3 => Self::C,
            0x4 => Self::D,
            0x5 => Self::Idx,
            0x6 => Self::Idy,
            0x7 => Self::Address,
            0x8 => Self::IndexedAddress,
            0x9 => Self::IndirectAddress,
            0xA => Self::IndirectIndexedAddress,
            0xB => Self::IndexedIndirectAddress,
            0xC => Self::IndexedPointer,
            0xD => Self::IndirectPointer,
            0xE => Self::IndirectIndexedPointer,
            0xF => Self::IndexedIndirectPointer,
            _ => panic!("Illegal destination value: {}", destination),
        }
    }

    fn get_source_from_instruction(instruction: u16) -> Self {
        let source = (instruction & 0x0F00) >> 8;

        match source {
            0x0 => Self::Immediate,
            0x1 => Self::A,
            0x2 => Self::B,
            0x3 => Self::C,
            0x4 => Self::D,
            0x5 => Self::Idx,
            0x6 => Self::Idy,
            0x7 => Self::Address,
            0x8 => Self::IndexedAddress,
            0x9 => Self::IndirectAddress,
            0xA => Self::IndirectIndexedAddress,
            0xB => Self::IndexedIndirectAddress,
            0xC => Self::IndexedPointer,
            0xD => Self::IndirectPointer,
            0xE => Self::IndirectIndexedPointer,
            0xF => Self::IndexedIndirectPointer,
            _ => panic!("Illegal source value: {}", source),
        }
    }
}

pub struct CPU {
    pub enable: bool,
    pub waiting_for_interrupt: bool,
    pub memory_controller: MemoryController,
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub index_x: u16,
    pub index_y: u16,
    pub status: u8, // szpc_oi
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            enable: false,
            waiting_for_interrupt: false,
            memory_controller: MemoryController::new(),
            program_counter: 0x0000,
            stack_pointer: 0x0000,
            index_x: 0x0000,
            index_y: 0x0000,
            status: 0b0000_0000,
            a: 0x0000,
            b: 0x0000,
            c: 0x0000,
            d: 0x0000,
        }
    }

    pub fn reset(&mut self) {
        self.enable = true;
        self.waiting_for_interrupt = false;
        self.memory_controller.reset();
        self.program_counter = self.memory_controller.read16(RESET_VECTOR);
        self.stack_pointer = 0x0000;
        self.index_x = 0x0000;
        self.index_y = 0x0000;
        self.status = 0b0000_0000;
        self.a = 0x0000;
        self.b = 0x0000;
        self.c = 0x0000;
        self.d = 0x0000;
    }

    pub fn print_state(&self) {
        println!("PC: 0x{:04X}", self.program_counter);
        println!("SP: 0x{:04X}", self.stack_pointer);
        println!("IDX: 0x{:04X}", self.index_x);
        println!("IDY: 0x{:04X}", self.index_y);
        println!("FLG: 0b{:08b}", self.status);
        println!("A: 0x{:04X}", self.a);
        println!("B: 0x{:04X}", self.b);
        println!("C: 0x{:04X}", self.c);
        println!("D: 0x{:04X}", self.d);
    }

    pub fn process(&mut self, nmi: bool, irq: Option<u8>) {
        if !self.enable {
            return;
        }

        if self.waiting_for_interrupt {
            if nmi {
                self.waiting_for_interrupt = false;

                self.set_interrupt_disable_flag(true);

                self.program_counter = self.memory_controller.read16(NMI_VECTOR);
            }

            if let Some(irq_code) = irq {
                self.waiting_for_interrupt = false;

                if self.get_interrupt_disable_flag() {
                    return;
                }

                self.set_interrupt_disable_flag(true);

                self.program_counter = self.memory_controller.read16(self.memory_controller.read16(IRQ_VECTOR) as usize + (irq_code as usize * 2));
            }

            return;
        }

        let instruction = self.fetch16();

        let operation = Operation::get_operation_from_instruction(instruction);
        let byte_mode = (instruction & 0x0040) != 0;
        let lo_hi = (instruction & 0x0080) != 0;
        let destination = Location::get_destination_from_instruction(instruction);
        let source = Location::get_source_from_instruction(instruction);

        match operation {
            Operation::Mov => {
                if byte_mode {
                    self.execute_mov8(lo_hi, destination, source);
                } else {
                    self.execute_mov16(destination, source);
                }
            },
            Operation::Adc => {
                if byte_mode {
                    self.execute_adc8(lo_hi, destination, source);
                } else {
                    self.execute_adc16(destination, source);
                }
            },
            Operation::Sbc => {
                if byte_mode {
                    self.execute_sbc8(lo_hi, destination, source);
                } else {
                    self.execute_sbc16(destination, source);
                }
            },
            Operation::Stp => {
                self.enable = false;
            },
            Operation::Rst => {
                self.reset();
            },
            Operation::Nop => (),
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

    fn get_interrupt_disable_flag(&self) -> bool {
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

    fn set_interrupt_disable_flag(&mut self, flag: bool) {
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

    fn set_al(&mut self, value: u8) {
        self.a = self.a & 0xFF00 | value as u16;
    }

    fn set_ah(&mut self, value: u8) {
        self.a = self.a & 0x00FF | (value as u16) << 8;
    }

    fn set_bl(&mut self, value: u8) {
        self.b = self.b & 0xFF00 | value as u16;
    }

    fn set_bh(&mut self, value: u8) {
        self.b = self.b & 0x00FF | (value as u16) << 8;
    }

    fn set_cl(&mut self, value: u8) {
        self.c = self.c & 0xFF00 | value as u16;
    }

    fn set_ch(&mut self, value: u8) {
        self.c = self.c & 0x00FF | (value as u16) << 8;
    }

    fn set_dl(&mut self, value: u8) {
        self.d = self.d & 0xFF00 | value as u16;
    }

    fn set_dh(&mut self, value: u8) {
        self.d = self.d & 0x00FF | (value as u16) << 8;
    }

    fn set_idxl(&mut self, value: u8) {
        self.index_x = self.index_x & 0xFF00 | value as u16;
    }

    fn set_idxh(&mut self, value: u8) {
        self.index_x = self.index_x & 0x00FF | (value as u16) << 8;
    }

    fn set_idyl(&mut self, value: u8) {
        self.index_y = self.index_y & 0xFF00 | value as u16;
    }

    fn set_idyh(&mut self, value: u8) {
        self.index_y = self.index_y & 0x00FF | (value as u16) << 8;
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

    fn subtract_with_carry16(&mut self, lhs: u16, rhs: u16, carry: bool) -> u16 {
        let (result, borrow) = lhs.borrowing_sub(rhs, !carry);

        self.set_flags_from_value16(result);
        self.set_carry_flag(!borrow);
        self.set_overflow_flag(lhs & 0x8000 == rhs & 0x8000 && lhs & 0x8000 != result & 0x8000);

        result
    }

    fn subtract_with_carry8(&mut self, lhs: u8, rhs: u8, carry: bool) -> u8 {
        let (result, borrow) = lhs.borrowing_sub(rhs, !carry);

        self.set_flags_from_value8(result);
        self.set_carry_flag(!borrow);
        self.set_overflow_flag(lhs & 0x80 == rhs & 0x80 && lhs & 0x80 != result & 0x80);

        result
    }

    fn execute_mov16(&mut self, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch16(),
            Location::A => self.a,
            Location::B => self.b,
            Location::C => self.c,
            Location::D => self.d,
            Location::Idx => self.index_x,
            Location::Idy => self.index_y,
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
        };

        self.set_flags_from_value16(source_value);

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                self.a = source_value;
            },
            Location::B => {
                self.b = source_value;
            },
            Location::C => {
                self.c = source_value;
            },
            Location::D => {
                self.d = source_value;
            },
            Location::Idx => {
                self.index_x = source_value;
            },
            Location::Idy => {
                self.index_y = source_value;
            },
            Location::Address => {
                let destination_address = self.fetch_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                self.memory_controller.write16(destination_address, source_value);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                self.memory_controller.write16(destination_address, source_value);
            },
        }
    }

    fn execute_mov8(&mut self, lo_hi: bool, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch8(),
            Location::A => {
                if lo_hi {
                    (self.a >> 8) as u8
                } else {
                    self.a as u8
                }
            },
            Location::B => {
                if lo_hi {
                    (self.b >> 8) as u8
                } else {
                    self.b as u8
                }
            },
            Location::C => {
                if lo_hi {
                    (self.c >> 8) as u8
                } else {
                    self.c as u8
                }
            },
            Location::D => {
                if lo_hi {
                    (self.d >> 8) as u8
                } else {
                    self.d as u8
                }
            },
            Location::Idx => {
                if lo_hi {
                    (self.index_x >> 8) as u8
                } else {
                    self.index_x as u8
                }
            },
            Location::Idy => {
                if lo_hi {
                    (self.index_y >> 8) as u8
                } else {
                    self.index_y as u8
                }
            },
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
        };

        self.set_flags_from_value8(source_value);

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                if lo_hi {
                    self.set_ah(source_value);
                } else {
                    self.set_al(source_value);
                }
            },
            Location::B => {
                if lo_hi {
                    self.set_bh(source_value);
                } else {
                    self.set_bl(source_value);
                }
            },
            Location::C => {
                if lo_hi {
                    self.set_ch(source_value);
                } else {
                    self.set_cl(source_value);
                }
            },
            Location::D => {
                if lo_hi {
                    self.set_dh(source_value);
                } else {
                    self.set_dl(source_value);
                }
            },
            Location::Idx => {
                if lo_hi {
                    self.set_idxh(source_value);
                } else {
                    self.set_idxl(source_value);
                }
            },
            Location::Idy => {
                if lo_hi {
                    self.set_idyh(source_value);
                } else {
                    self.set_idyl(source_value);
                }
            },
            Location::Address => {
                let destination_address = self.fetch_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                self.memory_controller.write8(destination_address, source_value);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                self.memory_controller.write8(destination_address, source_value);
            },
        }
    }

    fn execute_adc16(&mut self, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch16(),
            Location::A => self.a,
            Location::B => self.b,
            Location::C => self.c,
            Location::D => self.d,
            Location::Idx => self.index_x,
            Location::Idy => self.index_y,
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
        };

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                self.a = self.add_with_carry16(self.a, source_value, self.get_carry_flag());
            },
            Location::B => {
                self.b = self.add_with_carry16(self.b, source_value, self.get_carry_flag());
            },
            Location::C => {
                self.c = self.add_with_carry16(self.c, source_value, self.get_carry_flag());
            },
            Location::D => {
                self.d = self.add_with_carry16(self.d, source_value, self.get_carry_flag());
            },
            Location::Idx => {
                self.index_x = self.add_with_carry16(self.index_x, source_value, self.get_carry_flag());
            },
            Location::Idy => {
                self.index_y = self.add_with_carry16(self.index_y, source_value, self.get_carry_flag());
            },
            Location::Address => {
                let destination_address = self.fetch_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.add_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
        }
    }

    fn execute_adc8(&mut self, lo_hi: bool, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch8(),
            Location::A => {
                if lo_hi {
                    (self.a >> 8) as u8
                } else {
                    self.a as u8
                }
            },
            Location::B => {
                if lo_hi {
                    (self.b >> 8) as u8
                } else {
                    self.b as u8
                }
            },
            Location::C => {
                if lo_hi {
                    (self.c >> 8) as u8
                } else {
                    self.c as u8
                }
            },
            Location::D => {
                if lo_hi {
                    (self.d >> 8) as u8
                } else {
                    self.d as u8
                }
            },
            Location::Idx => {
                if lo_hi {
                    (self.index_x >> 8) as u8
                } else {
                    self.index_x as u8
                }
            },
            Location::Idy => {
                if lo_hi {
                    (self.index_y >> 8) as u8
                } else {
                    self.index_y as u8
                }
            },
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
        };

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                let result = self.add_with_carry8(self.a as u8, source_value, self.get_carry_flag());

                if lo_hi {
                    self.set_ah(result);
                } else {
                    self.set_al(result);
                }
            },
            Location::B => {
                let result = self.add_with_carry8(self.b as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_bh(result);
                } else {
                    self.set_bl(result);
                }
            },
            Location::C => {
                let result = self.add_with_carry8(self.c as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_ch(result);
                } else {
                    self.set_cl(result);
                }
            },
            Location::D => {
                let result = self.add_with_carry8(self.d as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_dh(result);
                } else {
                    self.set_dl(result);
                }
            },
            Location::Idx => {
                let result = self.add_with_carry8(self.index_x as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_idxh(result);
                } else {
                    self.set_idxl(result);
                }
            },
            Location::Idy => {
                let result = self.add_with_carry8(self.index_y as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_idyh(result);
                } else {
                    self.set_idyl(result);
                }
            },
            Location::Address => {
                let destination_address = self.fetch_address();
                
                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.add_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
        }
    }

    fn execute_sbc16(&mut self, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch16(),
            Location::A => self.a,
            Location::B => self.b,
            Location::C => self.c,
            Location::D => self.d,
            Location::Idx => self.index_x,
            Location::Idy => self.index_y,
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read16(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read16(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read16(source_address)
            },
        };

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                self.a = self.subtract_with_carry16(self.a, source_value, self.get_carry_flag());
            },
            Location::B => {
                self.b = self.subtract_with_carry16(self.b, source_value, self.get_carry_flag());
            },
            Location::C => {
                self.c = self.subtract_with_carry16(self.c, source_value, self.get_carry_flag());
            },
            Location::D => {
                self.d = self.subtract_with_carry16(self.d, source_value, self.get_carry_flag());
            },
            Location::Idx => {
                self.index_x = self.subtract_with_carry16(self.index_x, source_value, self.get_carry_flag());
            },
            Location::Idy => {
                self.index_y = self.subtract_with_carry16(self.index_y, source_value, self.get_carry_flag());
            },
            Location::Address => {
                let destination_address = self.fetch_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.subtract_with_carry16(self.memory_controller.read16(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write16(destination_address, result);
            },
        }
    }

    fn execute_sbc8(&mut self, lo_hi: bool, destination: Location, source: Location) {
        let source_value = match source {
            Location::Immediate => self.fetch8(),
            Location::A => {
                if lo_hi {
                    (self.a >> 8) as u8
                } else {
                    self.a as u8
                }
            },
            Location::B => {
                if lo_hi {
                    (self.b >> 8) as u8
                } else {
                    self.b as u8
                }
            },
            Location::C => {
                if lo_hi {
                    (self.c >> 8) as u8
                } else {
                    self.c as u8
                }
            },
            Location::D => {
                if lo_hi {
                    (self.d >> 8) as u8
                } else {
                    self.d as u8
                }
            },
            Location::Idx => {
                if lo_hi {
                    (self.index_x >> 8) as u8
                } else {
                    self.index_x as u8
                }
            },
            Location::Idy => {
                if lo_hi {
                    (self.index_y >> 8) as u8
                } else {
                    self.index_y as u8
                }
            },
            Location::Address => {
                let source_address = self.fetch_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndexedAddress => {
                let source_address = self.fetch_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectAddress => {
                let source_address = self.fetch_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedAddress => {
                let source_address = self.fetch_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectAddress => {
                let source_address = self.fetch_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedPointer => {
                let source_address = self.get_pointer_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndirectPointer => {
                let source_address = self.get_pointer_indirect_address();

                self.memory_controller.read8(source_address)
            },
            Location::IndirectIndexedPointer => {
                let source_address = self.get_pointer_indirect_indexed_address();
                
                self.memory_controller.read8(source_address)
            },
            Location::IndexedIndirectPointer => {
                let source_address = self.get_pointer_indexed_indirect_address();
                
                self.memory_controller.read8(source_address)
            },
        };

        match destination {
            Location::Immediate => (), // NOP
            Location::A => {
                let result = self.subtract_with_carry8(self.a as u8, source_value, self.get_carry_flag());

                if lo_hi {
                    self.set_ah(result);
                } else {
                    self.set_al(result);
                }
            },
            Location::B => {
                let result = self.subtract_with_carry8(self.b as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_bh(result);
                } else {
                    self.set_bl(result);
                }
            },
            Location::C => {
                let result = self.subtract_with_carry8(self.c as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_ch(result);
                } else {
                    self.set_cl(result);
                }
            },
            Location::D => {
                let result = self.subtract_with_carry8(self.d as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_dh(result);
                } else {
                    self.set_dl(result);
                }
            },
            Location::Idx => {
                let result = self.subtract_with_carry8(self.index_x as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_idxh(result);
                } else {
                    self.set_idxl(result);
                }
            },
            Location::Idy => {
                let result = self.subtract_with_carry8(self.index_y as u8, source_value, self.get_carry_flag());
                
                if lo_hi {
                    self.set_idyh(result);
                } else {
                    self.set_idyl(result);
                }
            },
            Location::Address => {
                let destination_address = self.fetch_address();
                
                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedAddress => {
                let destination_address = self.fetch_indexed_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectAddress => {
                let destination_address = self.fetch_indirect_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectIndexedAddress => {
                let destination_address = self.fetch_indirect_indexed_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedIndirectAddress => {
                let destination_address = self.fetch_indexed_indirect_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedPointer => {
                let destination_address = self.get_pointer_indexed_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectPointer => {
                let destination_address = self.get_pointer_indirect_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndirectIndexedPointer => {
                let destination_address = self.get_pointer_indirect_indexed_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
            Location::IndexedIndirectPointer => {
                let destination_address = self.get_pointer_indexed_indirect_address();

                let result = self.subtract_with_carry8(self.memory_controller.read8(destination_address), source_value, self.get_carry_flag());

                self.memory_controller.write8(destination_address, result);
            },
        }
    }
}