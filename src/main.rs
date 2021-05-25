mod cpu;
mod ppu;

use crate::cpu::CPU;

fn main() {
    println!("Hello, world!");

    let mut cpu = CPU::new();
    let program = vec![
        0x10, // write to...
        0x00, // register A
        0xFF, // the value.

        0xFF,

        0x20, // write to (memory)...
        0x02, // the region in memory pt. 1
        0x00, // the region in memory pt. 2
        0x00, // register A,

        0xFF,

        0x20, // write to (memory)...
        0x05, // the region in memory pt. 1
        0xFF, // the region in memory pt. 2
        0x00, // register A,

        // mov 1 to register B
        0x10,
        0x01,
        0x01,

        0x40, // jump if...
        0x01, // register B is true...
        0x80,
        0x00, // this region in memory

        0xFF,

        0x00, // HALT
    ];

    cpu.load(program);
    cpu.run();
}
