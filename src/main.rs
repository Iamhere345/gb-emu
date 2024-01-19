use std::fs::File;
use std::io::prelude::*;

use bus::*;
use cpu::*;
use instructions::*;
use registers::*;

mod cpu;
mod registers;
mod bus;
mod instructions;

fn main() {
    
	let bytes = include_bytes!("../test.gb");

	let mut bus: Bus = Bus::new();
	let mut cpu: CPU = CPU::new(&mut bus);

	for (addr, byte) in bytes.iter().enumerate() {
		cpu.bus.write_byte(addr.try_into().unwrap(), *byte);
	}

	let instructions = get_unprefixed_instructions();
	let prefixed_instructions = get_prefixed_instructions();

	cpu.pc = 0x0;

	cpu.cycle(&instructions, &prefixed_instructions);
	cpu.cycle(&instructions, &prefixed_instructions);
	cpu.cycle(&instructions, &prefixed_instructions);
	cpu.cycle(&instructions, &prefixed_instructions);
	println!("{}", cpu.registers.get_16bit_reg(Register16Bit::HL));
	println!("{}", cpu.registers.get_16bit_reg(Register16Bit::BC));
	cpu.cycle(&instructions, &prefixed_instructions);


	println!("{}", cpu.registers.get_16bit_reg(Register16Bit::HL));

}
