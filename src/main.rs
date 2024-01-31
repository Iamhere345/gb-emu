use bus::*;
use cpu::cpu::CPU;
use cpu::registers::*;

use std::io::{prelude::*, BufWriter};
use std::fs::File;

mod cpu;
mod bus;

fn main() {
    
	let bytes = include_bytes!("../tests/cpu_instrs/individual/06-ld r,r.gb");

	let mut bus: Bus = Bus::new();
	let mut cpu: CPU = CPU::new(&mut bus);

	for (addr, byte) in bytes.iter().enumerate() {
		cpu.bus.write_byte(addr.try_into().unwrap(), *byte);
	}

	let log = File::create("emu.log").expect("unable to open log file");
	let mut log_writer = BufWriter::new(&log);

	cpu.pc = 0xFF;

	loop {
		let result = cpu.cycle();

		let a = cpu.registers.get_8bit_reg(Register8Bit::A);
		let f = cpu.registers.get_8bit_reg(Register8Bit::F);
		let b = cpu.registers.get_8bit_reg(Register8Bit::B);
		let c = cpu.registers.get_8bit_reg(Register8Bit::C);
		let d = cpu.registers.get_8bit_reg(Register8Bit::D);
		let e = cpu.registers.get_8bit_reg(Register8Bit::E);
		let h = cpu.registers.get_8bit_reg(Register8Bit::H);
		let l = cpu.registers.get_8bit_reg(Register8Bit::L);
		let sp = cpu.registers.get_16bit_reg(Register16Bit::SP);
		let pc = cpu.pc;
		let pc0 = cpu.bus.read_byte(cpu.pc);
		let pc1 = cpu.bus.read_byte(cpu.pc + 1);
		let pc2 = cpu.bus.read_byte(cpu.pc + 2);
		let pc3 = cpu.bus.read_byte(cpu.pc + 3);

		write!(&mut log_writer, "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n", a,f,b,c,d,e,h,l,sp,pc,pc0,pc1,pc2,pc3).expect("E");

		if result == 1 {
			break;
		}
	}

}
