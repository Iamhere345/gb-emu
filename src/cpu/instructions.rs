#![allow(non_snake_case)]
#![allow(unused_variables)]
use std::vec;

use lazy_static::lazy_static;

use super::cpu::CPU;
use super::registers::*;
use super::decode::*;

pub struct Instruction {
	pub opcodes: Vec<u8>,
	pub cycles: u16,
	pub mnemonic: &'static str,
	pub exec: fn(cpu: &mut CPU, opcode: u8, cycles: &mut u16)
}

impl Instruction {
	fn new(opcodes: Vec<u8>, cycles: u16, mnemonic: &'static str, exec: fn(cpu: &mut CPU, opcode: u8, cycles: &mut u16)) -> Self {
		Self {
			opcodes: opcodes,
			cycles: cycles,
			mnemonic: mnemonic,
			exec: exec
		}
	}
}
lazy_static!{
	pub static ref INSTRUCTIONS: Vec<Instruction> = vec![
		Instruction::new(vec![0x00], 4, "NOP", NOP),

		// any 8-bit instruction with [HL] takes longer

		// misc instructions
		Instruction::new(vec![0x0], 4, "NOP", NOP),
		Instruction::new(vec![0x2F], 4, "CPL", CPL),
		Instruction::new(vec![0x3F], 4, "CCF", CCF),
		Instruction::new(vec![0x37], 4, "SCF", SCF),
		Instruction::new(vec![0x27], 4, "DAA", DAA),

		// jump / subroutine instructions
		Instruction::new(vec![0x20, 0x30, 0x18, 0x28, 0x38], 12, "JR (cond), e8", JR_COND_E8),

		// memory instructions
		Instruction::new(vec![0x01, 0x11, 0x21, 0x31], 12, "LD r16, imm16", LD_R16_IMM),
		Instruction::new(vec![0x06, 0x16, 0x26, 0x0E, 0x1E, 0x2E, 0x3E], 8, "LD r8, imm8", LD_R8_IMM),
		Instruction::new(vec![0x36], 12, "LD r8, [HL]", LD_R8_IMM),
		Instruction::new(vec![0x02, 0x12, 0x22, 0x32], 8, "LD [r16mem], A", LD_R16MEM_A),
		Instruction::new(vec![0x0A, 0x1A, 0x2A, 0x3A], 8, "LD A, [r16mem]", LD_A_R16MEM),

		// stack instructions
		Instruction::new(vec![0x08], 20, "LD [a16], SP", LD_A16_SP),

		// arithmetic instructions
		Instruction::new(vec![0x03, 0x13, 0x23, 0x33], 8, "INC r16", INC_R16),
		Instruction::new(vec![0x0B, 0x1B, 0x2B, 0x3B], 8, "DEC r16", DEC_R16),
		Instruction::new(vec![0x04, 0x14, 0x24, 0x0C, 0x1C, 0x2C, 0x3C], 4, "INC r8", INC_R8),
		Instruction::new(vec![0x34], 12, "INC [HL],", INC_R8),
		Instruction::new(vec![0x05, 0x15, 0x25, 0x0D, 0x1D, 0x2D, 0x3D], 4, "DEC r8", DEC_R8),
		Instruction::new(vec![0x35], 12, "DEC [HL]", DEC_R8),

		Instruction::new(vec![0x09, 0x19, 0x29, 0x39], 8, "ADD HL, r16", ADD_HL_R16),

		// shift, rotate and bit instructions
		Instruction::new(vec![0x07], 4, "RLCA", RLCA_RRCA),
		Instruction::new(vec![0x0F], 4, "RRCA", RLCA_RRCA),

		Instruction::new(vec![0x17], 4, "RLA", RLA_RRA),
		Instruction::new(vec![0x1F], 4, "RRA", RLA_RRA),

	];

	pub static ref PREFIXED_INSTRUCTIONS: Vec<Instruction> = vec![];

}

// ! misc instructions

/*

MNEMONIC: NOP
OPCODES: 0x00
DESC: No-op; does nothing
FLAGS: - - - -

*/
fn NOP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	cpu.pc += 1;
}

/*

MNEMONIC: CPL
OPCODES: 0x2F
DESC: Flips the A register (!A)
FLAGS: - N H -

*/
fn CPL(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let new_value = !cpu.registers.get_8bit_reg(Register8Bit::A);

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_flag(Flag::N, true);
	cpu.registers.set_flag(Flag::H, true);

	cpu.pc += 1;
}

/*

MNEMONIC: CCF
OPCODES: 0x3F
DESC: Flips the carry flag, zeros N and H flags
FLAGS: - 0 0 !C

*/
fn CCF(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	cpu.registers.set_flag(Flag::N, false);
	cpu.registers.set_flag(Flag::H, false);
	cpu.registers.set_flag(Flag::C, !cpu.registers.get_flag(Flag::C));
	

	cpu.pc += 1;
}

/*

MNEMONIC: SCF
OPCODES: 0x3F
DESC: Sets the carry flag, zeros N and H flags
FLAGS: - 0 0 1

*/
fn SCF(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	cpu.registers.set_flag(Flag::N, false);
	cpu.registers.set_flag(Flag::H, false);
	cpu.registers.set_flag(Flag::C, true);
	

	cpu.pc += 1;
}

/*

MNEMONIC: DAA
OPCODES: 0x00
DESC: For Binary-coded decimal numbers (https://blog.ollien.com/posts/gb-daa/)
FLAGS: - - - -

*/
fn DAA(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let mut offset: u8 = 0;
	let mut should_carry = false;

	let old_value = cpu.registers.get_8bit_reg(Register8Bit::A);

	let half_carry = cpu.registers.get_flag(Flag::H);
	let carry = cpu.registers.get_flag(Flag::C);
	let subtract = cpu.registers.get_flag(Flag::N);

	if (subtract == false && old_value & 0xF > 0x09) || half_carry == false {
		offset |= 0x06;
	}

	if (subtract == false && old_value > 0x99) || carry == true {
		offset |= 0x60;
		should_carry = true;
	}

	let new_value = match subtract {
		true => old_value.wrapping_sub(offset),
		false => old_value.wrapping_add(offset)
	};

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_flag(Flag::Z, new_value == 0);
	cpu.registers.set_flag(Flag::H, false);
	cpu.registers.set_flag(Flag::C, should_carry);

	cpu.pc += 1;
}


// ! jump / subroutine instructions
/*

MNEMONIC: JR (cond), e8 (signed address offset)
OPCODES: 0x(1-3)8, 0x(2-3)0
DESC: performs an unconditional jump relative to pc (i.e pc += e8), depending on if the condition is true (or present).
Takes 8 cycles if no jump is performed, 12 cycles if it is performed.
FLAGS: - - - -

*/
fn JR_COND_E8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let jump: bool;

	if (opcode >> 3) == 3 {
		jump = true;
	} else {
		let cond = Cond::new((opcode & 0x18) >> 3);

		let z = cpu.registers.get_flag(Flag::Z);
		let c = cpu.registers.get_flag(Flag::C);

		match cond {
			Cond::nz => if !z 	{ jump = true; } else { jump = false; },
			Cond::z => 	if z 	{ jump = true; } else { jump = false; },
			Cond::nc => if !c 	{ jump = true; } else { jump = false; },
			Cond::c => 	if c 	{ jump = true; } else { jump = false; }
		}
	}

	if !jump {
		cpu.pc += 1;
		*cycles = 8;
		return;
	}

	let offset: i8 = get_imm8(cpu) as i8;

	let offset_unsigned: u16 = offset.abs().try_into().unwrap();

	if offset.is_negative() {
		cpu.pc -= offset_unsigned;
	} else {
		cpu.pc += offset_unsigned;
	}

}

// ! memory instructions

/*

MNEMONIC: LD r16, imm16
OPCODES: 0x(0-3)1 
DESC: Loads a 16-bit immediate number into a 16-bit register (little endian)
FLAGS: - - - -

*/
fn LD_R16_IMM(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
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

MNEMONIC: LD r8, imm8
OPCODES: 0x(0-3)6, 0x(0-3)E
DESC: loads an 8 bit immediate into r8
FLAGS: - - - -

*/
fn LD_R8_IMM(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let dest = Register8Bit::from_r8(opcode >> 4);

	cpu.pc += 1;
	let src = cpu.bus.read_byte(cpu.pc);

	cpu.set_8bit_reg(dest, src);

	cpu.pc += 1;
}

/*

MNEMONIC: LD [r16mem], A
OPCODES: 0x(0-3)2
DESC: Loads A into the memory location pointed to by r16mem
if r16mem is HL+ / HL- it will inc or dec HL after the operation
FLAGS: - - - -

*/
fn LD_R16MEM_A(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
	let dest_info = Register16Bit::from_r16mem(opcode >> 4);
	let dest = cpu.registers.get_16bit_reg(dest_info.0);

	cpu.bus.write_byte(dest, cpu.registers.get_8bit_reg(Register8Bit::A));

	// postinc or postdec
	cpu.registers.set_16bit_reg(dest_info.0, (dest as i16 + dest_info.1 as i16) as u16);

	cpu.pc += 1;

}

/*

MNEMONIC: LD A, [r16mem]
OPCODES: 0x(0-3)A
DESC: Loads the data pointed to by r16mem into A.
if r16mem is HL+ / HL- it will inc or dec HL after the operation
FLAGS: - - - -

*/
fn LD_A_R16MEM(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
	let src_info = Register16Bit::from_r16mem(opcode >> 4);
	let src = cpu.registers.get_16bit_reg(src_info.0);

	cpu.set_8bit_reg(Register8Bit::A, cpu.bus.read_byte(src));

	// postinc or postdec
	cpu.registers.set_16bit_reg(src_info.0, (src as i16 + src_info.1 as i16) as u16);

	cpu.pc += 1;

}

// ! stack instructions

/*

MNEMONIC: LD [a16], SP
OPCODES: 0x08
DESC: Loads a 16-bit immediate number into a 16-bit register (little endian)
FLAGS: - - - -

*/
fn LD_A16_SP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	cpu.pc += 1;
	let lo_addr = cpu.bus.read_byte(cpu.pc);

	cpu.pc += 1;
	let hi_addr = cpu.bus.read_byte(cpu.pc);

	let addr: u16 = ((hi_addr as u16) << 8) | lo_addr as u16;

	let sp = cpu.registers.get_16bit_reg(Register16Bit::SP);
	cpu.bus.write_byte(addr, (sp & 0xFF) as u8);
	cpu.bus.write_byte(addr + 1, (sp >> 8) as u8);

	cpu.pc += 1;

}


// ! arithmetic instructions

/*

MNEMONIC: INC r16
OPCODES: 0x(0-3)3 
DESC: Increments a 16-bit register; doesnt set flags
FLAGS: - - - -

*/
fn INC_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
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
fn DEC_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

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
fn INC_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);

	let old_carry = cpu.registers.get_flag(Flag::C);

	let old_value = cpu.get_8bit_reg(reg);
	let new_value = cpu.add_8bit(old_value, 1);
	cpu.set_8bit_reg(reg, new_value);

	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc += 1;

}

/*

MNEMONIC: DEC r8
OPCODES: 0x(0-3)5, 0x(0-3)D
DESC: Decrements an 8-bit register
FLAGS: Z 1 H -

*/
fn DEC_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);

	let old_carry = cpu.registers.get_flag(Flag::C);

	let old_value = cpu.get_8bit_reg(reg);
	let new_value = cpu.sub_8bit(old_value, 1);
	cpu.set_8bit_reg(reg, new_value);

	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc += 1;

}

/*

MNEMONIC: ADD HL, r16
OPCODES: 0x(0-3)9
DESC: HL = HL + r16
FLAGS: - 0 H C

*/
fn ADD_HL_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let lhs = cpu.registers.get_16bit_reg(Register16Bit::HL);
	let rhs = cpu.registers.get_16bit_reg(Register16Bit::from_r16(opcode >> 4));

	let old_z = cpu.registers.get_flag(Flag::Z);

	let new_hl = cpu.add_16bit(lhs, rhs);
	cpu.registers.set_16bit_reg(Register16Bit::HL, new_hl);

	cpu.registers.set_flag(Flag::Z, old_z);

	cpu.pc += 1;
}

// ! 8-bit shift, rotate and bit instructions

/*

MNEMONIC: RLCA, RRCA
OPCODES: 0x07, 0x0F
DESC: Shifts the A register to the left. The carry bit is set to the shifted out bit
FLAGS: 0 0 0 C

*/
fn RLCA_RRCA(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);

	if opcode >> 3 == 0 {
		// RRCA
		(new_value, carry) = cpu.registers.get_8bit_reg(Register8Bit::A).overflowing_shl(1);
	} else {
		//RLCA
		(new_value, carry) = cpu.registers.get_8bit_reg(Register8Bit::A).overflowing_shr(1);
	}

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);

	cpu.pc += 1;
}

/*

MNEMONIC: RLA, RRA
OPCODES: 0x00
DESC: Shifts the A register to the left. Wraps around to the carry bit, then carry bit is set to the shifted out bit.
FLAGS: 0 0 0 C

*/
fn RLA_RRA(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);
	let is_rla = (opcode & 0x8) >> 3 == 0;

	if is_rla {
		(new_value, carry) = cpu.registers.get_8bit_reg(Register8Bit::A).overflowing_shl(1);
	} else {
		(new_value, carry) = cpu.registers.get_8bit_reg(Register8Bit::A).overflowing_shr(1);
	}

	let old_carry = match is_rla {
		true => cpu.registers.get_flag(Flag::C) as u8,
		false => (cpu.registers.get_flag(Flag::C) as u8) << 7
	};

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value | old_carry);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);

	cpu.pc += 1;
}