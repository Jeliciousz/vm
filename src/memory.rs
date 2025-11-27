pub struct MemoryController<'memcontrol> {
    mappings: Vec<Mapping<'memcontrol>>,
}

impl<'memcontrol> MemoryController<'memcontrol> {
    pub fn new() -> Self {
        Self {
            mappings: Vec::<Mapping>::new()
        }
    }

    pub fn add_mapping(&mut self, start_index: usize, device: &'memcontrol mut dyn MappedDevice) -> Result<(), String> {
        match self.mappings.iter().find(|mapping| start_index >= mapping.start_index && start_index < mapping.end_index || start_index < mapping.start_index && start_index + device.size() >= mapping.start_index) {
            Some(_) => Err(String::from("New mapping overlaps existing mapping")),
            None => {
                self.mappings.push(Mapping {
                    start_index,
                    end_index: start_index + device.size(),
                    device,
                });

                Ok(())
            }
        }
    }

    pub fn read_8(&self, index: usize) -> u8 {
        match self.mappings.iter().find(|mapping| index >= mapping.start_index && index < mapping.end_index) {
            Some(mapping) => {
                let mapped_index = index - mapping.start_index;
                mapping.device.read_8(mapped_index)
            },
            None => 0x00,
        }
    }

    pub fn read_16(&self, index: usize) -> u16 {
        match self.mappings.iter().find(|mapping| index >= mapping.start_index && index < mapping.end_index) {
            Some(mapping) => {
                let mapped_index = index - mapping.start_index;
                mapping.device.read_16(mapped_index)
            },
            None => 0x00,
        }
    }

    pub fn write_8(&mut self, index: usize, value: u8) {
        match self.mappings.iter_mut().find(|mapping| index >= mapping.start_index && index < mapping.end_index) {
            Some(mapping) => {
                let mapped_index = index - mapping.start_index;
                mapping.device.write_8(mapped_index, value);
            },
            None => (),
        }
    }

    pub fn write_16(&mut self, index: usize, value: u16) {
        match self.mappings.iter_mut().find(|mapping| index >= mapping.start_index && index < mapping.end_index) {
            Some(mapping) => {
                let mapped_index = index - mapping.start_index;
                mapping.device.write_16(mapped_index, value);
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
    start_index: usize,
    end_index: usize,
    device: &'mapping mut dyn MappedDevice,
}

pub trait MappedDevice {
    fn read_8(&self, index: usize) -> u8;
    fn read_16(&self, index: usize) -> u16;
    fn size(&self) -> usize;
    fn write_8(&mut self, index: usize, value: u8);
    fn write_16(&mut self, index: usize, value: u16);
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

    pub fn read_bytes(&mut self, index: usize, count: usize) -> &[u8] {
        &self.memory[index..index + count]
    }

    pub fn write_bytes(&mut self, index: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[index + i] = *byte;
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for RAM {
    fn read_8(&self, index: usize) -> u8 {
        if index >= self.memory.len() {
            return 0x00;
        }
        self.memory[index]
    }

    fn read_16(&self, index: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if index < self.memory.len() {
            value &= self.memory[index] as u16;
        }
        if index + 1 < self.memory.len() {
            value &= (self.memory[index + 1] as u16) << 8;
        }

        value
    }

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn write_8(&mut self, index: usize, value: u8) {
        if index >= self.memory.len() {
            return;
        }
        self.memory[index] = value;
    }

    fn write_16(&mut self, index: usize, value: u16) {
        if index < self.memory.len() {
            self.memory[index] = value as u8;
        }
        if index + 1 < self.memory.len() {
            self.memory[index + 1] = (value >> 8) as u8;
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

    pub fn read_bytes(&mut self, index: usize, count: usize) -> &[u8] {
        &self.memory[index..index + count]
    }

    pub fn write_bytes(&mut self, index: usize, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[index + i] = *byte;
        }
    }

    pub fn fill(&mut self, value: u8) {
        self.memory.fill(value);
    }
}

impl MappedDevice for ROM {
    fn read_8(&self, index: usize) -> u8 {
        if index >= self.memory.len() {
            return 0x00;
        }
        self.memory[index]
    }

    fn read_16(&self, index: usize) -> u16 {
        let mut value: u16 = 0x0000;

        if index < self.memory.len() {
            value &= self.memory[index] as u16;
        }
        if index + 1 < self.memory.len() {
            value &= (self.memory[index + 1] as u16) << 8;
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