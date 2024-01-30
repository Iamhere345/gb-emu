use core::num;

use super::registers::*;
use super::cpu::CPU;

#[allow(non_camel_case_types)]
pub enum Cond {
	nz,
	z,
	nc,
	c,
}

impl Cond {
	pub fn new(num: u8) -> Self {
		match num {
			0 => Self::nz,
			1 => Self::z,
			2 => Self::nc,
			3 => Self::c,
			_ => panic!("invalid condition flag")
		}
	}
}

pub fn get_imm8(cpu: &mut CPU) -> u8 {
	cpu.pc += 1;
	cpu.bus.read_byte(cpu.pc)
}

pub fn get_imm16(cpu: &mut CPU) -> u16 {
	cpu.pc += 1;
	let lo_byte = cpu.bus.read_byte(cpu.pc);

	cpu.pc += 1;
	let hi_byte = cpu.bus.read_byte(cpu.pc);

	((hi_byte as u16) << 8) | lo_byte as u16
}