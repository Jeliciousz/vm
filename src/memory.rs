use crate::cpu::ADDRESS_BUS_WIDTH;

pub const ADDRESS_SPACE: usize = 2_usize.pow(ADDRESS_BUS_WIDTH);
pub const MAP_BLOCK_SIZE: usize = 0x1000; // 4 KiB
pub const MAP_BLOCKS: usize = ADDRESS_SPACE / MAP_BLOCK_SIZE;

pub struct MemoryController {
    blocks: [Option<usize>; MAP_BLOCKS],
    mappings: Vec<Mapping>,
}

impl MemoryController {
    pub fn new() -> Self {
        Self {
            blocks: [None; MAP_BLOCKS],
            mappings: vec![],
        }
    }

    // Returns index of device mapping or an error
    pub fn map_device(&mut self, first_block: usize, blocks: usize, device: Box<dyn MappedDevice>) -> Result<usize, String> {
        for block in first_block..first_block + blocks {
            if self.blocks[block] != None {
                return Err(format!("Block {} is already mapped to another device", block));
            }
        }

        self.mappings.push(Mapping {
            offset: first_block * MAP_BLOCK_SIZE,
            device,
         });

        let mapping_index = self.mappings.len() - 1;

        for block in first_block..first_block + blocks {
            self.blocks[block] = Some(mapping_index);
        }

        Ok(mapping_index)
    }

    pub fn unmap_device(&mut self, mapping_index: usize) -> Result<(), String> {
        if mapping_index >= self.mappings.len() {
            return Err(format!("Index {} is out-of-bounds", mapping_index));
        }

        self.mappings.swap_remove(mapping_index);

        // Because swap_remove removes an element and replaces it with the last element in the vector, we need to update all the blocks that pointed to the last element
                
        let last_mapping_index = self.mappings.len(); // The current length of the vector equals the index of what the last element in the vector *used* to be

        for block in &mut self.blocks {
            // Remove all indecies to the element we just removed
            if *block == Some(mapping_index) {
                *block = None;
            }
            // Change all indecies that used to point to the last element to point to the element it just replaced
            if *block == Some(last_mapping_index) {
                *block = Some(mapping_index);
            }
        }

        Ok(())
    }

    pub fn get_device(&self, mapping_index: usize) -> Result<&dyn MappedDevice, String> {
        if mapping_index >= self.mappings.len() {
            return Err(format!("Index {} is out-of-bounds", mapping_index));
        }

        Ok(self.mappings[mapping_index].device.as_ref())
    }

    pub fn get_device_mut(&mut self, mapping_index: usize) -> Result<&mut dyn MappedDevice, String> {
        if mapping_index >= self.mappings.len() {
            return Err(format!("Index {} is out-of-bounds", mapping_index));
        }

        Ok(self.mappings[mapping_index].device.as_mut())
    }

    pub fn read_8(&self, address: usize) -> u8 {
        match self.blocks[address / MAP_BLOCK_SIZE] {
            Some(mapping_index) => {
                let translated_address = address - self.mappings[mapping_index].offset;

                self.mappings[mapping_index].device.read_8(translated_address)
            },
            None => 0x00,
        }
    }

    pub fn read_16(&self, address: usize) -> u16 {
        match self.blocks[address / MAP_BLOCK_SIZE] {
            Some(mapping_index) => {
                let translated_address = address - self.mappings[mapping_index].offset;
                
                self.mappings[mapping_index].device.read_16(translated_address)
            },
            None => 0x00,
        }
    }

    pub fn write_8(&mut self, address: usize, value: u8) {
        match self.blocks[address / MAP_BLOCK_SIZE] {
            Some(mapping_index) => {
                let translated_address = address - self.mappings[mapping_index].offset;
                
                self.mappings[mapping_index].device.write_8(translated_address, value)
            },
            None => (),
        }
    }

    pub fn write_16(&mut self, address: usize, value: u16) {
        match self.blocks[address / MAP_BLOCK_SIZE] {
            Some(mapping_index) => {
                let translated_address = address - self.mappings[mapping_index].offset;
                
                self.mappings[mapping_index].device.write_16(translated_address, value)
            },
            None => (),
        }
    }

    pub fn reset(&mut self) {
        for mapping in self.mappings.iter_mut() {
            mapping.device.reset();
        }
    }
}

struct Mapping {
    offset: usize,
    device: Box<dyn MappedDevice>,
}

pub trait MappedDevice {
    fn peek_bytes(&mut self, address: usize, count: usize) -> &[u8];
    fn poke_bytes(&mut self, address: usize, bytes: &[u8]);
    fn size(&self) -> usize;
    fn read_8(&self, address: usize) -> u8;
    fn read_16(&self, address: usize) -> u16;
    fn write_8(&mut self, address: usize, value: u8);
    fn write_16(&mut self, address: usize, value: u16);
    fn reset(&mut self);
}

pub struct RAM {
    memory: Box<[u8]>,
}

impl RAM {
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: vec![0x00_u8; capacity].into_boxed_slice(),
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for RAM {
    fn peek_bytes(&mut self, address: usize, count: usize) -> &[u8] {
        &self.memory[address..address + count]
    }

    fn poke_bytes(&mut self, address: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[address + i] = *byte;
        }
    }

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn read_8(&self, address: usize) -> u8 {
        if address >= self.memory.len() {
            return 0x00;
        }
        self.memory[address]
    }

    fn read_16(&self, address: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if address < self.memory.len() {
            value |= self.memory[address] as u16;
        }
        if address + 1 < self.memory.len() {
            value |= (self.memory[address + 1] as u16) << 8;
        }

        value
    }

    fn write_8(&mut self, address: usize, value: u8) {
        if address >= self.memory.len() {
            return;
        }
        self.memory[address] = value;
    }

    fn write_16(&mut self, address: usize, value: u16) {
        if address < self.memory.len() {
            self.memory[address] = value as u8;
        }
        if address + 1 < self.memory.len() {
            self.memory[address + 1] = (value >> 8) as u8;
        }
    }

    fn reset(&mut self) {
        self.memory.fill(0x00);
    }
}

pub struct ROM {
    memory: Box<[u8]>,
}

impl ROM {
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: vec![0x00_u8; capacity].into_boxed_slice(),
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for ROM {
    fn peek_bytes(&mut self, address: usize, count: usize) -> &[u8] {
        &self.memory[address..address + count]
    }

    fn poke_bytes(&mut self, address: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[address + i] = *byte;
        }
    }

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn read_8(&self, address: usize) -> u8 {
        if address >= self.memory.len() {
            return 0x00;
        }
        self.memory[address]
    }

    fn read_16(&self, address: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if address < self.memory.len() {
            value |= self.memory[address] as u16;
        }
        if address + 1 < self.memory.len() {
            value |= (self.memory[address + 1] as u16) << 8;
        }

        value
    }

    fn write_8(&mut self, _: usize, _: u8) {}
    fn write_16(&mut self, _: usize, _: u16) {}
    fn reset(&mut self) {}
}