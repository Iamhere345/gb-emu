#![allow(non_snake_case)]
#![allow(unused_variables)]
use std::vec;

use crate::cpu::CPU;
use crate::registers::*;

pub struct Instruction {
	pub opcodes: Vec<u8>,
	pub cycles: u16,
	pub mnemonic: &'static str,
	pub exec: fn(cpu: &mut CPU, opcode: u8, cycles: u16)
}

impl Instruction {
	fn new(opcodes: Vec<u8>, cycles: u16, mnemonic: &'static str, exec: fn(cpu: &mut CPU, opcode: u8, cycles: u16)) -> Self {
		Self {
			opcodes: opcodes,
			cycles: cycles,
			mnemonic: mnemonic,
			exec: exec
		}
	}
}


pub fn get_unprefixed_instructions() -> Vec<Instruction> {
	let instructions: Vec<Instruction> = vec![
		Instruction::new(vec![0x00], 4, "NOP", NOP),

		Instruction::new(vec![0x01, 0x11, 0x21, 0x31], 12, "LD r16, imm16", LD_R16_IMM),
		Instruction::new(vec![0x02, 0x12, 0x22, 0x32], 8, "LD [r16mem], A", LD_R16MEM_A),

		Instruction::new(vec![0x03, 0x13, 0x23, 0x33], 8, "INC r16", INC_R16),
		Instruction::new(vec![0x0B, 0x1B, 0x2B, 0x3B], 8, "DEC r16", DEC_R8),

		Instruction::new(vec![0x04, 0x14, 0x24, 0x34], 12, "INC r8 (L)", INC_R8),
		Instruction::new(vec![0x0C, 0x1C, 0x2C, 0x3C], 4, "INC r8, (R)", INC_R8),
		Instruction::new(vec![0x05, 0x15, 0x25, 0x35], 12, "DEC r8 (L)", DEC_R8),
		Instruction::new(vec![0x0D, 0x1D, 0x2D, 0x3D], 4, "DEC r8 (R)", DEC_R8),
	];

	instructions
}

pub fn get_prefixed_instructions() -> Vec<Instruction> {
	let prefixed_instructions: Vec<Instruction> = Vec::new();

	prefixed_instructions
}


// ! misc instructions

/*
fn NOP(cpu: &mut CPU, opcode: u8, cycles: u16) {
	
}
*/

/*

MNEMONIC: NOP
OPCODES: [0x00] 
DESC: No-op; does nothing
FLAGS: - - - -

*/
fn NOP(cpu: &mut CPU, opcode: u8, cycles: u16) {
	cpu.pc += 1;
}

// ! memory instructions

/*

MNEMONIC: LD r16, imm16
OPCODES: 0x(0-3)1 
DESC: Loads a 16-bit immediate number into a 16-bit register
FLAGS: - - - -

*/
fn LD_R16_IMM(cpu: &mut CPU, opcode: u8, cycles: u16) {
	let dest = Register16Bit::from_r16(opcode >> 4);

	cpu.pc += 1;
	let lo_src = cpu.bus.read_byte(cpu.pc);

	cpu.pc += 1;
	let hi_src = cpu.bus.read_byte(cpu.pc);

	let src: u16 = ((hi_src as u16) << 8) | lo_src as u16;

	cpu.registers.set_16bit_reg(dest, src);

	cpu.pc += 1;

}

/*

MNEMONIC: LD [r16mem], A
OPCODES: 
DESC: 0x(0-3)2 Loads A into the memory location pointed to by r16mem
if r16mem is HL+ / HL- it will inc or dec HL after the operation
FLAGS: - - - -

*/
fn LD_R16MEM_A(cpu: &mut CPU, opcode: u8, cycles: u16) {
	
	let dest_info = Register16Bit::from_r16mem(opcode >> 4);
	let dest = cpu.registers.get_16bit_reg(dest_info.0);

	cpu.bus.write_byte(dest, cpu.registers.get_8bit_reg(Register8Bit::A));

	// postinc or postdec
	cpu.registers.set_16bit_reg(dest_info.0, (dest as i16 + dest_info.1 as i16) as u16);

	cpu.pc += 1;

}

// ! arithmetic instructions

/*

MNEMONIC: INC r16
OPCODES: 0x(0-3)3 
DESC: Increments a 16-bit register; doesnt set flags
FLAGS: - - - -

*/
fn INC_R16(cpu: &mut CPU, opcode: u8, cycles: u16) {
	
	let reg = Register16Bit::from_r16(opcode >> 4);

	let old_flags = cpu.registers.get_8bit_reg(Register8Bit::F);

	let new_value = cpu.add_16bit(cpu.registers.get_16bit_reg(reg), 1);
	cpu.registers.set_16bit_reg(reg, new_value);

	// restore flags
	cpu.registers.set_8bit_reg(Register8Bit::F, old_flags);

	cpu.pc += 1;
}

/*

MNEMONIC: DEC r16
OPCODES: 0x(0-3)B 
DESC: Decrements a 16-bit registers; doesnt set flags
FLAGS: - - - -

*/
fn DEC_R16(cpu: &mut CPU, opcode: u8, cycles: u16) {

	let reg = Register16Bit::from_r16(opcode >> 4);

	let old_flags = cpu.registers.get_8bit_reg(Register8Bit::F);

	let new_value = cpu.sub_16bit(cpu.registers.get_16bit_reg(reg), 1);
	cpu.registers.set_16bit_reg(reg, new_value);

	// restore flags
	cpu.registers.set_8bit_reg(Register8Bit::F, old_flags);

	cpu.pc += 1;
}

/*

MNEMONIC: INC r8
OPCODES: 0x(0-3)4, 0x(0-3)C 
DESC: Increments an 8-bit register (or [HL]); sets flags
FLAGS: Z 0 H -

*/
fn INC_R8(cpu: &mut CPU, opcode: u8, cycles: u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);

	let old_carry = cpu.registers.get_flag(Flag::C);

	if reg == Register8Bit::HL {
		// increment the value the memory location stored in HL, set flags
		let target = cpu.registers.get_16bit_reg(Register16Bit::HL);
		let new_value = cpu.add_8bit(cpu.bus.read_byte(target), 1);
		
		cpu.bus.write_byte(target, new_value);
	} else {
		let new_value = cpu.add_8bit(cpu.registers.get_8bit_reg(reg), 1);
		cpu.registers.set_8bit_reg(reg, new_value);
	}

	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc += 1;

}

/*

MNEMONIC: DEC r8
OPCODES: 0x(0-3)5, 0x(0-3)D
DESC: Decrements an 8-bit register
FLAGS: Z 1 H -

*/
fn DEC_R8(cpu: &mut CPU, opcode: u8, cycles: u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);

	let old_carry = cpu.registers.get_flag(Flag::C);

	if reg == Register8Bit::HL {
		// decrement the value the memory location stored in HL, set flags
		let target = cpu.registers.get_16bit_reg(Register16Bit::HL);
		let new_value = cpu.sub_8bit(cpu.bus.read_byte(target), 1);
		
		cpu.bus.write_byte(target, new_value);
	} else {
		let new_value = cpu.sub_8bit(cpu.registers.get_8bit_reg(reg), 1);
		cpu.registers.set_8bit_reg(reg, new_value);
	}

	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc += 1;

}