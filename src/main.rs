mod cpu;
mod memory;

use cpu::CPU;
use memory::{RAM, ROM};

const RAM_CAPACITY: usize = 0x1000; // 4 KiB
const ROM_CAPACITY: usize = 0x8000; // 32 KiB
const RAM_BLOCKS: usize = RAM_CAPACITY / memory::MAP_BLOCK_SIZE;
const ROM_BLOCKS: usize = ROM_CAPACITY / memory::MAP_BLOCK_SIZE;

const RAM_FIRST_ADDRESS: usize = 0x0000;
const ROM_FIRST_ADDRESS: usize = 0x8000;
const RAM_FIRST_BLOCK: usize = RAM_FIRST_ADDRESS / memory::MAP_BLOCK_SIZE;
const ROM_FIRST_BLOCK: usize = ROM_FIRST_ADDRESS / memory::MAP_BLOCK_SIZE;

fn main() {
    let mut cpu = CPU::new();

    cpu.memory_controller.map_device(RAM_FIRST_BLOCK, RAM_BLOCKS, Box::new(RAM::new(RAM_CAPACITY))).expect("Should not overlap");
    let rom_index = cpu.memory_controller.map_device(ROM_FIRST_BLOCK, ROM_BLOCKS, Box::new(ROM::new(ROM_CAPACITY))).expect("Should not overlap");
    let rom = cpu.memory_controller.get_device_mut(rom_index).expect("This is a known index");

    // Set reset vector to the first address in ROM
    let low_byte = ROM_FIRST_ADDRESS as u8;
    let high_byte = (ROM_FIRST_ADDRESS >> 8) as u8;

    rom.poke_bytes(cpu::RESET_VECTOR - ROM_FIRST_ADDRESS, &[low_byte, high_byte]);
    rom.poke_bytes(0x0000, &[0x00, 0x80, 0xF0]);

    cpu.reset();

    println!("Program counter after reset: 0x{:04X}", cpu.program_counter);
    
    loop {
        cpu.step();
        println!("Accumulator after step: 0x{:02X}", cpu.accumulator);
    }
}
