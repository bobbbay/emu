use crate::ppu::PPU;

#[derive(Debug)]
pub struct CPU {
    pub registers: [u8; 4],
    pub memory: [u8; 0xFFFF],
    pub pc: u16,
    pub ppu: PPU,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: [0; 4],
            memory: [0; 0xFFFF],
            pc: 0,
            ppu: PPU::new(),
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        // Note: we initialize the pc here, which is where run() will start at.
        self.pc = 0x8000;
    }

    #[allow(unused_doc_comments)]
    pub fn run(&mut self) {
        loop {
            self.ppu.render(self.memory);
            self.memory = self.ppu.update_keys(self.memory);

            dbg!(self.memory[0x0100]);

            let opcode = self.mem_read(self.pc);
            self.pc += 1;

            match opcode {
                /// Halt
                0x00 => return,
                /// No-op
                0xFF => (),

                /// Load value into register; LOAD
                0x10 => {
                    let reg_index = self.mem_read_next_for_register_index();

                    let value = self.mem_read_next();

                    self.registers[reg_index] = value;
                }

                /// Load from another register
                0x11 => {
                    let reg_index = self.mem_read_next_for_register_index();

                    let content = self.registers[self.mem_read_next_as_usize()];

                    self.registers[reg_index] = content;
                }

                /// Load to a register from memory
                0x12 => {
                    let reg_index = self.mem_read_next_for_register_index();

                    let address = self.mem_read_u16_be_next();

                    self.registers[reg_index] = self.mem_read(address);
                }

                /// Store 8 bits to a region in memory from a register
                0x20 => {
                    let address = self.mem_read_u16_be_next();

                    let reg_index = self.mem_read_next_for_register_index();

                    self.mem_write(address, self.registers[reg_index]);
                }

                /// Compare $A == $B storing the result in $C
                0x30 => {
                    let reg1 = self.registers[self.mem_read_next_for_register_index()];

                    let reg2 = self.registers[self.mem_read_next_for_register_index()];

                    // We create a `u8` from a `bool` - on true, it becomes 1, and on false it becomes 0.
                    self.registers[self.mem_read_next_for_register_index()] =
                        u8::from(reg1 == reg2);
                }

                /// Compare $A == 0xB and store the result in $C
                0x31 => {
                    let reg = self.registers[self.mem_read_next_for_register_index()];

                    let value = self.mem_read(self.pc);
                    self.pc += 1;

                    // We create a `u8` from a `bool` - on true, it becomes 1, and on false it becomes 0.
                    self.registers[self.mem_read_next_for_register_index()] =
                        u8::from(reg == value);
                }

                /// Compare $A > $B storing the result in $C
                0x32 => {
                    let reg1 = self.registers[self.mem_read_next_for_register_index()];

                    let reg2 = self.registers[self.mem_read_next_for_register_index()];

                    // We create a `u8` from a `bool` - on true, it becomes 1, and on false it becomes 0.
                    self.registers[self.mem_read_next_for_register_index()] = u8::from(reg1 > reg2);
                }

                /// Compare $A < 0xB and store the result in $C
                0x33 => {
                    let reg1 = self.registers[self.mem_read_next_for_register_index()];

                    let reg2 = self.registers[self.mem_read_next_for_register_index()];

                    // We create a `u8` from a `bool` - on true, it becomes 1, and on false it becomes 0.
                    self.registers[self.mem_read_next_for_register_index()] = u8::from(reg1 < reg2);
                }

                /// If $A is true, jump to 0xB in the program counter
                0x40 => {
                    let reg = self.registers[self.mem_read_next_for_register_index()];

                    // We don't use `mem_read_u16_be_next()` here for efficiency reasons - there
                    // would be no need to increment the program counter if we do end up changing it.
                    // If not, we'll increment it manually.
                    let target = self.mem_read_u16_be(self.pc);

                    if reg == 1 {
                        self.pc = target;
                    } else {
                        self.pc += 2;
                    }
                }

                /// Increment $A
                0x50 => {
                    let reg_index = self.mem_read_next_for_register_index();

                    self.registers[reg_index] += 1;
                }

                /// Decrement $A
                0x51 => {
                    let reg_index = self.mem_read_next_for_register_index();

                    self.registers[reg_index] -= 1;
                }

                /// Perform $A + $B and store the result in $C
                0x52 => {
                    let reg1_index = self.mem_read_next_for_register_index();
                    let reg2_index = self.mem_read_next_for_register_index();
                    let reg3_index = self.mem_read_next_for_register_index();

                    // We use .wrapping_add() here to denote that if we overflow, wrap to 0.
                    self.registers[reg3_index] =
                        self.registers[reg1_index].wrapping_add(self.registers[reg2_index]);
                }

                /// Perform $A - $B and store the result in $C
                0x53 => {
                    let reg1_index = self.mem_read_next_for_register_index();
                    let reg2_index = self.mem_read_next_for_register_index();
                    let reg3_index = self.mem_read_next_for_register_index();

                    self.registers[reg3_index] =
                        self.registers[reg1_index].wrapping_sub(self.registers[reg2_index]);
                }

                /// Perform $A + 0xB and store the result in $C
                0x54 => {
                    let reg1_index = self.mem_read_next_for_register_index();
                    let val2 = self.mem_read_next();
                    let reg3_index = self.mem_read_next_for_register_index();

                    self.registers[reg3_index] = self.registers[reg1_index].wrapping_add(val2);
                }

                /// Perform $A - 0xB and store the result in $C
                0x55 => {
                    let reg1_index = self.mem_read_next_for_register_index();
                    let val2 = self.mem_read_next();
                    let reg3_index = self.mem_read_next_for_register_index();

                    self.registers[reg3_index] = self.registers[reg1_index].wrapping_sub(val2);
                }

                _ => unimplemented!(),
            }
        }
    }

    /// Reads 8 bits after `addr`
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    /// Reads the next 8 bits after self.pc and increments it respectively
    fn mem_read_next(&mut self) -> u8 {
        let res = self.mem_read(self.pc);
        self.pc += 1;

        res
    }

    /// Performs `mem_read_next()` but returns a safely casted usize
    fn mem_read_next_as_usize(&mut self) -> usize {
        self.mem_read_next() as usize
    }

    /// Performs `mem_read_next_as_usize()` but checks to make sure the supplied value is a valid
    /// register
    fn mem_read_next_for_register_index(&mut self) -> usize {
        // Note that since we use unsigned memory, there is no need to check if the value is larger
        // than 0.
        if self.mem_read(self.pc) <= 4 {
            self.mem_read_next_as_usize()
        } else {
            panic!("Invalid register: {}", self.mem_read(self.pc));
        }
    }

    /// Writes `data` to `addr`
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    /// Reads 16 bits after `pos`
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    /// Reads the next 16 bits in memory and increments self.pc respectively
    fn mem_read_u16_next(&mut self) -> u16 {
        let res = self.mem_read_u16(self.pc);
        self.pc += 1;

        res
    }

    /// Reads 16 bits after `pos` as Big Endian
    fn mem_read_u16_be(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (lo << 8) | hi as u16
    }

    /// Reads the next 16 bits in memory as Big Endian and increments self.pc respectively
    fn mem_read_u16_be_next(&mut self) -> u16 {
        let res = self.mem_read_u16_be(self.pc);
        self.pc += 2;

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_program() {
        let mut cpu = CPU::new();
        let program = vec![0x00];

        // We load the program in, which will add the opcode into memory and point the program
        // counter to the beginning.
        cpu.load(program);

        cpu.run();
    }

    #[test]
    fn test_load() {
        let mut cpu = CPU::new();
        let program = vec![
            0x10, 0x00, // $A (the register)
            0xFF, // 0xB (the value)
            0x00,
        ];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[0], 0xFF,)
    }

    #[test]
    fn test_load_from_register() {
        let mut cpu = CPU::new();
        let program = vec![
            0x11, 0x00, // $A (the register to write to)
            0x01, // $B (the register to read from)
            0x00,
        ];

        cpu.registers[1] = 0xFF;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[0], 0xFF,)
    }

    #[test]
    fn test_load_from_memory() {
        let mut cpu = CPU::new();
        let program = vec![
            0x12, 0x00, // $A (the register to write to)
            0x00, 0xAB, // 0xB (the region in memory to read from)
            0x00,
        ];

        cpu.mem_write(0x00AB, 0xFF);

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[0], 0xFF,)
    }

    #[test]
    fn test_store_to_mem() {
        let mut cpu = CPU::new();
        let program = vec![
            0x20, 0x00, 0xAB, // 0xB (the region in memory to write to)
            0x00, // $A (the register to read from)
            0x00,
        ];

        cpu.registers[0] = 0xFF;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.mem_read(0x00AB), 0xFF,)
    }

    #[test]
    fn test_compare_registers_true() {
        let mut cpu = CPU::new();
        let program = vec![
            0x30, 0x00, // $A (the first register to compare)
            0x01, // $B (the second register to compare)
            0x02, // $C (the register to store the result in)
            0x00,
        ];

        cpu.registers[0] = 100;
        cpu.registers[1] = 100;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 1,)
    }

    #[test]
    fn test_compare_registers_false() {
        let mut cpu = CPU::new();
        let program = vec![
            0x30, // See test_compare_registers_true()
            0x00, 0x01, 0x02, 0x00,
        ];

        cpu.registers[0] = 100;
        cpu.registers[1] = 200;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 0,)
    }

    #[test]
    fn test_compare_register_with_val_true() {
        let mut cpu = CPU::new();
        let program = vec![
            0x31, 0x00, // $A (the first register to compare)
            0xFF, // 0xB (the second value to compare)
            0x01, // $C (the register to store the result in)
            0x00,
        ];

        cpu.registers[0] = 0xFF;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[1], 1,)
    }

    #[test]
    fn test_compare_register_with_val_false() {
        let mut cpu = CPU::new();
        let program = vec![
            0x31, // See test_compare_register_with_val_true()
            0x00, 0xFF, 0x01, 0x00,
        ];

        cpu.registers[0] = 0xEE;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[1], 0,)
    }

    #[test]
    fn test_jump_if_true() {
        let mut cpu = CPU::new();
        let program = vec![
            0x40, 0x00, // $A (the register that we're checking)
            0x80, 0x05, // 0xB (the region in memory we're jumping the program counter to)
            0x00, // Blank, this will be skipped
            0x00,
        ];

        cpu.registers[0] = 1;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 32774,)
    }

    #[test]
    fn test_jump_if_false() {
        let mut cpu = CPU::new();
        let program = vec![
            0x40, // See test_jump_if_true()
            0x00, 0x80, 0x05, 0x00, // The program will reach here and end (address 32773)
            0x00, // The program will not reach here (address 32774)
        ];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 32773,)
    }

    #[test]
    fn test_increment_reg() {
        let mut cpu = CPU::new();
        let program = vec![
            0x50, 0x00, // $A (the register to increment)
            0x00,
        ];

        cpu.registers[0] = 5;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[0], 6,)
    }

    #[test]
    fn test_decrement_reg() {
        let mut cpu = CPU::new();
        let program = vec![
            0x51, 0x00, // $A (the register to increment)
            0x00,
        ];

        cpu.registers[0] = 5;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[0], 4,)
    }

    #[test]
    fn test_add_regs() {
        let mut cpu = CPU::new();
        let program = vec![
            0x52, 0x00, // $A (the first register to add)
            0x01, // $B (the second register to add)
            0x02, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 5;
        cpu.registers[1] = 5;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 10,)
    }

    #[test]
    fn test_add_regs_with_overflow() {
        let mut cpu = CPU::new();
        let program = vec![
            0x52, 0x00, // $A (the first register to add)
            0x01, // $B (the second register to add)
            0x02, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = u8::MAX;
        cpu.registers[1] = 5;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 4,)
    }

    #[test]
    fn test_sub_regs() {
        let mut cpu = CPU::new();
        let program = vec![
            0x53, 0x00, // $A (the first register to add)
            0x01, // $B (the second register to add)
            0x02, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 10;
        cpu.registers[1] = 9;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 1,)
    }

    #[test]
    fn test_sub_regs_with_overflow() {
        let mut cpu = CPU::new();
        let program = vec![
            0x53, 0x00, // $A (the first register to add)
            0x01, // $B (the second register to add)
            0x02, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 0;
        cpu.registers[1] = 5;

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.registers[2], 251,)
    }

    #[test]
    fn test_add_reg_to_val() {
        let mut cpu = CPU::new();
        let program = vec![
            0x54, 0x00, // $A (the first register to add)
            0x0A, // 0xB (the second value to add)
            0x01, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 5;

        cpu.load(program);
        cpu.run();

        // 5 + 10
        assert_eq!(cpu.registers[1], 15,)
    }

    #[test]
    fn test_add_reg_to_val_with_overflow() {
        let mut cpu = CPU::new();
        let program = vec![
            0x54, 0x00, // $A (the first register to add)
            0x0A, // 0xB (the second value to add)
            0x01, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 0xFF;

        cpu.load(program);
        cpu.run();

        // 255 + 10
        assert_eq!(cpu.registers[1], 9,)
    }

    #[test]
    fn test_sub_val_from_reg() {
        let mut cpu = CPU::new();
        let program = vec![
            0x55, 0x00, // $A (the first register to subtract)
            0x05, // 0xB (the second value to subtract)
            0x01, // $C (where to store the result)
            0x00,
        ];

        cpu.registers[0] = 15;

        cpu.load(program);
        cpu.run();

        // 15 - 5
        assert_eq!(cpu.registers[1], 10,)
    }

    #[test]
    fn test_sub_val_from_reg_with_overflow() {
        let mut cpu = CPU::new();
        let program = vec![
            0x55, // See test_sub_val_from_reg()
            0x00, 0x0A, 0x01, 0x00,
        ];

        cpu.registers[0] = 0;

        cpu.load(program);
        cpu.run();

        // 0 - 10
        assert_eq!(cpu.registers[1], 246,)
    }
}
