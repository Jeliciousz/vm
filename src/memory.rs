pub const ADDRESS_SPACE: usize = 0x10000;
pub const MAP_BLOCK_SIZE: usize = 0x1000;
pub const MAP_BLOCKS: usize = ADDRESS_SPACE / MAP_BLOCK_SIZE;

pub struct MemoryController<'memcontrol> {
    blocks: [Option<usize>; MAP_BLOCKS],
    mappings: Vec<Mapping<'memcontrol>>,
}

impl<'memcontrol> MemoryController<'memcontrol> {
    pub fn new() -> Self {
        Self {
            blocks: [None; MAP_BLOCKS],
            mappings: vec![],
        }
    }

    pub fn map_device(&mut self, first_block: usize, blocks: usize, device: &'memcontrol mut dyn MappedDevice) -> Result<(), String> {
        for block in first_block..first_block + blocks {
            if self.blocks[block] != None {
                return Err(format!("Block {} is already mapped to another device", block));
            }
        }

        match self.mappings.iter().find(|mapping| std::ptr::eq(mapping.device, device)) {
            Some(_) => Err(String::from("Device is already mapped")),
            None => {
                self.mappings.push(Mapping {
                    offset: first_block * MAP_BLOCK_SIZE,
                    device,
                });

                let mapping_index = self.mappings.len() - 1;

                for block in first_block..first_block + blocks {
                    self.blocks[block] = Some(mapping_index);
                }

                Ok(())
            },
        }
    }

    pub fn unmap_device(&mut self, device: &'memcontrol mut dyn MappedDevice) -> Result<(), String> {
        match self.mappings.iter().enumerate().find(|(_, mapping)| std::ptr::eq(mapping.device, device)) {
            Some((i, _)) => {
                self.mappings.swap_remove(i);

                // Because swap_remove removes an element and replaces it with the last element in the vector, we need to update all the blocks that pointed to the last element
                
                let last_mapping_index = self.mappings.len(); // The current length of the vector equals the index of what the last element in the vector *used* to be

                for block in &mut self.blocks {
                    // Remove all indecies to the element we just removed
                    if *block == Some(i) {
                        *block = None;
                    }
                    // Change all indecies that used to point to the last element to point to the element it just replaced
                    if *block == Some(last_mapping_index) {
                        *block = Some(i);
                    }
                }

                Ok(())
            },
            None => Err(String::from("Device is not mapped")),
        }
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

struct Mapping<'mapping> {
    device: &'mapping mut dyn MappedDevice,
    offset: usize,
}

pub trait MappedDevice {
    fn read_8(&self, address: usize) -> u8;
    fn read_16(&self, address: usize) -> u16;
    fn size(&self) -> usize;
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

    pub fn read_bytes(&mut self, address: usize, count: usize) -> &[u8] {
        &self.memory[address..address + count]
    }

    pub fn write_bytes(&mut self, address: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[address + i] = *byte;
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for RAM {
    fn read_8(&self, address: usize) -> u8 {
        if address >= self.memory.len() {
            return 0x00;
        }
        self.memory[address]
    }

    fn read_16(&self, address: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if address < self.memory.len() {
            value &= self.memory[address] as u16;
        }
        if address + 1 < self.memory.len() {
            value &= (self.memory[address + 1] as u16) << 8;
        }

        value
    }

    fn size(&self) -> usize {
        self.memory.len()
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

    pub fn read_bytes(&mut self, address: usize, count: usize) -> &[u8] {
        &self.memory[address..address + count]
    }

    pub fn write_bytes(&mut self, address: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[address + i] = *byte;
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for ROM {
    fn read_8(&self, address: usize) -> u8 {
        if address >= self.memory.len() {
            return 0x00;
        }
        self.memory[address]
    }

    fn read_16(&self, address: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if address < self.memory.len() {
            value &= self.memory[address] as u16;
        }
        if address + 1 < self.memory.len() {
            value &= (self.memory[address + 1] as u16) << 8;
        }

        value
    }

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn write_8(&mut self, _: usize, _: u8) {}
    fn write_16(&mut self, _: usize, _: u16) {}
    fn reset(&mut self) {}
}