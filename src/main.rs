mod cpu;
mod memory;

use cpu::CPU;
use memory::{RAM, ROM};

const RAM_CAPACITY: usize = memory::MAP_BLOCK_SIZE;
const ROM_CAPACITY: usize = memory::MAP_BLOCK_SIZE * 8;

const RAM_FIRST_BLOCK: usize = 0x0000 / memory::MAP_BLOCK_SIZE;
const RAM_BLOCKS: usize = RAM_CAPACITY / memory::MAP_BLOCK_SIZE;

const ROM_FIRST_BLOCK: usize = 0x8000 / memory::MAP_BLOCK_SIZE;
const ROM_BLOCKS: usize = ROM_CAPACITY / memory::MAP_BLOCK_SIZE;

fn main() {
    let mut cpu = CPU::new();

    let mut ram = RAM::new(RAM_CAPACITY);
    let mut rom = ROM::new(ROM_CAPACITY);

    cpu.memory_controller.map_device(RAM_FIRST_BLOCK, RAM_BLOCKS, &mut ram).expect("RAM hasn't been mapped yet and shouldn't overlap");
    cpu.memory_controller.map_device(ROM_FIRST_BLOCK, ROM_BLOCKS, &mut rom).expect("ROM hasn't been mapped yet and shouldn't overlap");

    cpu.reset();
}
