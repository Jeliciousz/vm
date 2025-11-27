mod cpu;
mod memory;

use cpu::CPU;
use memory::{RAM, ROM};

const RAM_START_INDEX: usize = 0x0000;
const RAM_CAPACITY: usize = 0x1000;
const ROM_START_INDEX: usize = 0x8000;
const ROM_CAPACITY: usize = 0x8000;

fn main() {
    let mut cpu = CPU::new();

    let mut ram = RAM::new(RAM_CAPACITY);
    let mut rom = ROM::new(ROM_CAPACITY);

    cpu.memory_controller.add_mapping(RAM_START_INDEX, &mut ram).expect("Should not overlap");
    cpu.memory_controller.add_mapping(ROM_START_INDEX, &mut rom).expect("Should not overlap");

    cpu.reset();
}
