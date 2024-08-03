#![allow(non_snake_case)]
#![allow(unused_variables)]
use std::vec;

use lazy_static::lazy_static;

use super::CPU;
use super::registers::*;

#[allow(non_camel_case_types)]
#[derive(Debug)]
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
	cpu.pc = cpu.pc.wrapping_add(1);
	cpu.bus.borrow().read_byte(cpu.pc)
}

pub fn get_imm16(cpu: &mut CPU) -> u16 {
	cpu.pc = cpu.pc.wrapping_add(1);
	let lo_byte = cpu.bus.borrow().read_byte(cpu.pc);

	cpu.pc = cpu.pc.wrapping_add(1);
	let hi_byte = cpu.bus.borrow().read_byte(cpu.pc);

	((hi_byte as u16) << 8) | lo_byte as u16
}

fn set_debug_str(cpu: &mut CPU, from: &str, to: String) {
	cpu.last_instruction = cpu.last_instruction.replace(from, to.as_str());
}

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

		Instruction::new(vec![0xFB], 4, "EI", EI),
		Instruction::new(vec![0xF3], 4, "DI", DI),

		Instruction::new(vec![0x76], 4, "HALT", HALT),
		Instruction::new(vec![0x10], 4, "STOP", STOP),

		// jump / subroutine instructions
		Instruction::new(vec![0x20, 0x30, 0x18, 0x28, 0x38], 12, "JR (cond), e8", JR_COND_E8),
		Instruction::new(vec![0xC3, 0xC2, 0xD2, 0xCA, 0xDA], 16, "JP (cond), a16", JP_COND_A16),
		Instruction::new(vec![0xE9], 4, "JP HL", JP_HL),
		Instruction::new(vec![0xC4, 0xD4, 0xCC, 0xDC, 0xCD], 24, "CALL (cond), a16", CALL_COND_A16),
		
		Instruction::new(vec![0xC7, 0xD7, 0xE7, 0xF7, 0xCF, 0xDF, 0xEF, 0xFF], 16, "RST vec", RST),

		Instruction::new(vec![0xC0, 0xD0, 0xC8, 0xD8], 20, "RET (cond)", RET_COND),
		Instruction::new(vec![0xC9], 16, "RET", RET_COND),
		Instruction::new(vec![0xD9], 16, "RETI", RET_COND),

		// memory instructions
		Instruction::new(vec![
			0x40, 0x50, 0x60, 0x70, 0x41, 0x51, 0x61, 0x71,
			0x42, 0x52, 0x62, 0x72, 0x43, 0x53, 0x63, 0x73,
			0x44, 0x54, 0x64, 0x74, 0x45, 0x55, 0x65, 0x75,
			0x46, 0x56, 0x66, 0x47, 0x57, 0x67, 0x77, 0x48, 
			0x58, 0x68, 0x78, 0x49, 0x59, 0x69, 0x79, 0x4A, 
			0x5A, 0x6A, 0x7A, 0x4B, 0x5B, 0x6B, 0x7B, 0x4C, 
			0x5C, 0x6C, 0x7C, 0x4D, 0x5D, 0x6D, 0x7D, 0x4E, 
			0x5E, 0x6E, 0x7E, 0x4F, 0x5F, 0x6F, 0x7F
		], 4, "LD r8, r8", LD_R8_R8),
		Instruction::new(vec![0x01, 0x11, 0x21, 0x31], 12, "LD r16, imm16", LD_R16_IMM),
		Instruction::new(vec![0x06, 0x16, 0x26, 0x0E, 0x1E, 0x2E, 0x3E], 8, "LD r8, imm8", LD_R8_IMM),
		Instruction::new(vec![0x36], 12, "LD r8, [HL]", LD_R8_IMM),
		Instruction::new(vec![0x02, 0x12, 0x22, 0x32], 8, "LD [r16mem], A", LD_R16MEM_A),
		Instruction::new(vec![0x0A, 0x1A, 0x2A, 0x3A], 8, "LD A, [r16mem]", LD_A_R16MEM),
		Instruction::new(vec![0xFA], 16, "LD A, [a16]", LD_A_A16),
		Instruction::new(vec![0xEA], 16, "LD [a16], A", LD_A16_A),

		Instruction::new(vec![0xF0], 12, "LDH A, [a8]", LDH_A_A8),
		Instruction::new(vec![0xE0], 12, "LDH [a8], A", LDH_A8_A),
		Instruction::new(vec![0xF2], 8, "LDH A, [C]", LDH_A_C),
		Instruction::new(vec![0xE2], 8, "LDH [C], A", LDH_C_A),

		// stack instructions
		Instruction::new(vec![0x08], 20, "LD [a16], SP", LD_A16_SP),
		Instruction::new(vec![0xC5, 0xD5, 0xE5, 0xF5], 16, "PUSH r16", PUSH_R16),
		Instruction::new(vec![0xC1, 0xD1, 0xE1, 0xF1], 12, "POP r16", POP_R16),
		Instruction::new(vec![0xF9], 8, "LD SP, HL", LD_SP_HL),
		Instruction::new(vec![0xF8], 12, "LD HL, SP + e8", LD_HL_SP_E8),
		Instruction::new(vec![0xE8], 16, "ADD SP, e8", ADD_SP_E8),

		// arithmetic instructions
		Instruction::new(vec![0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87], 4, "ADD A, r8", ADD_ADC),
		Instruction::new(vec![0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F], 4, "ADC A, r8", ADD_ADC),
		Instruction::new(vec![0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97], 4, "SUB A, r8", SUB_SBC),
		Instruction::new(vec![0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F], 4, "SBC A, r8", SUB_SBC),

		Instruction::new(vec![0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7], 4, "AND A, r8", AND),
		Instruction::new(vec![0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF], 4, "XOR A, r8", XOR),

		Instruction::new(vec![0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7], 4, "OR A, r8", OR),
		Instruction::new(vec![0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF], 4, "CP A, r8", CP),

		Instruction::new(vec![0xC6], 8, "ADD A, n8", ADD_ADC),
		Instruction::new(vec![0xCE], 8, "ADC A, n8", ADD_ADC),
		Instruction::new(vec![0xD6], 8, "SUB A, n8", SUB_SBC),
		Instruction::new(vec![0xDE], 8, "SBC A, n8", SUB_SBC),
		Instruction::new(vec![0xE6], 8, "AND A, n8", AND),
		Instruction::new(vec![0xEE], 8, "XOR A, n8", XOR),
		Instruction::new(vec![0xF6], 8, "OR A, n8", OR),
		Instruction::new(vec![0xFE], 8, "CP A, n8", CP),

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

	pub static ref PREFIXED_INSTRUCTIONS: Vec<Instruction> = vec![
		
		Instruction::new(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07], 8, "RLC r8", RLC_RRC_R8),
		Instruction::new(vec![0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F], 8, "RRC r8", RLC_RRC_R8),
		
		Instruction::new(vec![0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17], 8, "RL r8", RL_RR_R8),
		Instruction::new(vec![0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F], 8, "RR r8", RL_RR_R8),

		Instruction::new(vec![0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27], 8, "SLA r8", SLA_SRA_R8),
		Instruction::new(vec![0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F], 8, "SRA r8", SLA_SRA_R8),

		Instruction::new(vec![0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37], 8, "SWAP r8", SWAP_R8),
		Instruction::new(vec![0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F], 8, "SRL r8", SRL_R8),

		Instruction::new(vec![
			0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
			0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
			0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
			0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
		], 8, "BIT u3, r8", BIT_U3_R8),

		Instruction::new(vec![
			0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
			0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
			0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
			0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
		], 8, "RES u3, r8", RES_U3_R8),
		Instruction::new(vec![
			0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
			0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
			0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
			0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
		], 8, "SET u3, r8", SET_U3_R8),

	];

}

// ! misc instructions

/*

MNEMONIC: NOP
OPCODES: 0x00
DESC: No-op; does nothing
FLAGS: - - - -

*/
fn NOP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: EI
OPCODES: 0xFB
DESC: enables interrupt handling (IME = 1)
FLAGS: - - - -

*/
fn EI(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	// ? accuracy: the effects of EI are delayed by one instruction (important for the halt bug)
	//cpu.ei = 1;
	cpu.ime = true;

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: DI
OPCODES: 0xF4
DESC: disables interrupt handling (IME = 0)
FLAGS: - - - -

*/
fn DI(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	cpu.ime = false;

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: HALT
OPCODES: 0x76
DESC: Halts the CPU until an interrupt is serviced
FLAGS: - - - -

*/
fn HALT(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	// TODO halt bug

	cpu.halted = true;

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: STOP
OPCODES: 0x10
DESC: Unimplemented. Permenently halts the CPU
FLAGS: - - - -

*/
fn STOP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	//println!("!!! STOP INSTRUCTION IS UNIMPLEMENTED !!!");

	cpu.pc = cpu.pc.wrapping_add(1);
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

	cpu.pc = cpu.pc.wrapping_add(1);
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
	

	cpu.pc = cpu.pc.wrapping_add(1);
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
	

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: DAA
OPCODES: 0x27
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

	if (subtract == false && old_value & 0x0F > 0x09) || half_carry == true {
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

	cpu.pc = cpu.pc.wrapping_add(1);
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
		set_debug_str(cpu, "(cond),", "".to_string());
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

		set_debug_str(cpu, "(cond)", format!("{:?}", cond));
	}

	// the offset is fetched every time, even if the jump isn't executed
	let offset: i8 = get_imm8(cpu) as i8;

	set_debug_str(cpu, "e8", format!("{}", offset));

	let new_pc: u16 = (((cpu.pc.wrapping_add(1)) as i16).wrapping_add(offset as i16)) as u16;

	if !jump {
		cpu.pc = cpu.pc.wrapping_add(1);
		*cycles = 8;
		return;
	}

	cpu.pc = new_pc;

	/*
	let offset_unsigned: u16 = offset.abs().try_into().unwrap();

	if offset.is_negative() {
		cpu.pc = cpu.pc.wrapping_sub(offset_unsigned);
	} else {
		cpu.pc = cpu.pc.wrapping_add(offset_unsigned);
	}
	*/

}

/*

MNEMONIC: JP (cond), a16
OPCODES: 0xC3, 0x(C-D)2, 0x(C-D)A
DESC: Jumps to a 16-bit address if cond is true. Takes 16 cycles if cond is true, 12 if not.
FLAGS: - - - -

*/
fn JP_COND_A16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
	let jump: bool;

	if (opcode & 1) == 1 {
		jump = true;
		set_debug_str(cpu, "(cond),", "".to_string());
	} else {
		let cond = Cond::new((opcode & 0x18) >> 3);

		set_debug_str(cpu, "(cond)", format!("{:?}", cond));

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
		// instruction is 3 bytes long
		cpu.pc = cpu.pc.wrapping_add(3);
		*cycles = 12;
		return;
	}

	let new_addr = get_imm16(cpu);

	set_debug_str(cpu, "a16", format!("0x{:X}", new_addr));

	cpu.pc = new_addr;

}

/*

MNEMONIC: JP HL
OPCODES: 0xE9
DESC: Jumps to the address stored in HL
FLAGS: - - - -

*/
fn JP_HL(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let new_addr = cpu.registers.get_16bit_reg(Register16Bit::HL);
	cpu.pc = new_addr;
}

/*

MNEMONIC: Call (cond), a16
OPCODES: 0x(C-D)4, 0x(C-D)C, 0xCD
DESC: if the condition is met, push the current address onto the stack and jump to a16
FLAGS: - - - -

*/
fn CALL_COND_A16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let jump: bool;

	if (opcode & 1) == 1 {
		jump = true;
		set_debug_str(cpu, "(cond),", "".to_string());
	} else {
		let cond = Cond::new((opcode & 0x18) >> 3);

		set_debug_str(cpu, "(cond)", format!("{:?}", cond));

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
		// instruction is 3 bytes long
		cpu.pc = cpu.pc.wrapping_add(3);
		*cycles = 12;
		return;
	}

	let new_addr = get_imm16(cpu);
	
	cpu.pc = cpu.pc.wrapping_add(1);

	set_debug_str(cpu, "a16", format!("0x{:X} (push 0x{:X})", new_addr, cpu.pc));
	
	// push current address onto the stack
	cpu.push16(cpu.pc);
	
	cpu.pc = new_addr;

}

/*

MNEMONIC: RET (cond) / RETI
OPCODES: 0x(C-D)0, 0x(C-D)8, 0xC9, 0xD9
DESC: Pops the return address from the stack and puts it in pc. If the instruction is RETI
it will set IME first
FLAGS: - - - -

*/
fn RET_COND(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let jump: bool;

	// unconditional
	if (opcode & 1) == 1 {
		jump = true;
		set_debug_str(cpu, "(cond),", "".to_string());

		// RETI
		if opcode == 0xD9 {
			cpu.ime = true;
		}
		// conditional
	} else {
		let cond = Cond::new((opcode & 0x18) >> 3);

		set_debug_str(cpu, "(cond)", format!("{:?}", cond));

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
		cpu.pc = cpu.pc.wrapping_add(1);
		*cycles = 8;
		return;
	}

	// pop the return address from the stack
	let new_addr = cpu.pop16();

	// set pc to the return address
	cpu.pc = new_addr;

}

/*

MNEMONIC: RST vec
OPCODES: 0x(C-F)7, 0x(C-F)F
DESC: Calls the pre-defined reset vector
FLAGS: - - - -

*/
fn RST(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let vec = ((opcode >> 3) & 0x7) * 8;

	set_debug_str(cpu, "vec", format!("0x{:X}", vec));

	cpu.push16(cpu.pc.wrapping_add(1));

	cpu.pc = vec as u16;
}

// ! memory instructions

/*

MNEMONIC: LD r8, r8
OPCODES: 0x(4-7)(0-5), 0x(4-6)6, 0x(4-7)(7-F)
DESC: Loads the value from one register into another
FLAGS: - - - -

*/
fn LD_R8_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let src = Register8Bit::from_r8(opcode & 7);
	let dst = Register8Bit::from_r8((opcode >> 3) & 7);

	set_debug_str(cpu, "r8,", format!("{:?},", dst));
	set_debug_str(cpu, ", r8", format!(", {:?}", src));

	let value = cpu.get_8bit_reg(src);
	cpu.set_8bit_reg(dst, value);

	if src == Register8Bit::HL || dst == Register8Bit::HL {
		*cycles = 8;
	}

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LD r16, imm16
OPCODES: 0x(0-3)1 
DESC: Loads a 16-bit immediate number into a 16-bit register (little endian)
FLAGS: - - - -

*/
fn LD_R16_IMM(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let dest = Register16Bit::from_r16(opcode >> 4);
	let src = get_imm16(cpu);

	set_debug_str(cpu, "r16", format!("{:?}", dest));
	set_debug_str(cpu, "imm16", format!("0x{:X}", src));

	cpu.registers.set_16bit_reg(dest, src);

	cpu.pc = cpu.pc.wrapping_add(1);

}

/*

MNEMONIC: LD r8, imm8
OPCODES: 0x(0-3)6, 0x(0-3)E
DESC: loads an 8 bit immediate into r8
FLAGS: - - - -

*/
fn LD_R8_IMM(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let dest = Register8Bit::from_r8(opcode >> 3);
	let src = get_imm8(cpu);

	set_debug_str(cpu, "r8", format!("{:?}", dest));
	set_debug_str(cpu, "imm8", format!("0x{:X}", src));

	cpu.set_8bit_reg(dest, src);

	cpu.pc = cpu.pc.wrapping_add(1);
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

	if dest_info.1 == 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}", dest_info.0));
	} else if dest_info.1 > 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}+", dest_info.0));
	} else if dest_info.1 < 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}-", dest_info.0));
	}

	cpu.bus.borrow_mut().write_byte(dest, cpu.registers.get_8bit_reg(Register8Bit::A));

	// postinc or postdec
	cpu.registers.set_16bit_reg(dest_info.0, ((dest as i16).wrapping_add(dest_info.1 as i16)) as u16);

	cpu.pc = cpu.pc.wrapping_add(1);

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

	if src_info.1 == 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}", src_info.0));
	} else if src_info.1 > 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}+", src_info.0));
	} else if src_info.1 < 0 {
		set_debug_str(cpu, "r16mem", format!("{:?}-", src_info.0));
	}

	let new_a = cpu.bus.borrow().read_byte(src);

	cpu.set_8bit_reg(Register8Bit::A, new_a);

	// postinc or postdec
	cpu.registers.set_16bit_reg(src_info.0, ((src as i16).wrapping_add(src_info.1 as i16)) as u16);

	cpu.pc = cpu.pc.wrapping_add(1);

}

/*

MNEMONIC: LD A, [a16]
OPCODES: 0xFA
DESC: Loads the value stored in address a16 into A
FLAGS: - - - -

*/
fn LD_A_A16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr = get_imm16(cpu);
	let new_value = cpu.bus.borrow().read_byte(addr);

	set_debug_str(cpu, "a16", format!("0x{:X}", addr));

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LD [a16], A
OPCODES: 0xEA
DESC: Loads the value stored in A into address a16
FLAGS: - - - -

*/
fn LD_A16_A(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr = get_imm16(cpu);
	let a_value = cpu.registers.get_8bit_reg(Register8Bit::A);

	set_debug_str(cpu, "a16", format!("0x{:X}", addr));

	cpu.bus.borrow_mut().write_byte(addr, a_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LDH A, [a8]
OPCODES: 0xF0
DESC: Loads the value stored at 0xFF00+a8 into A
FLAGS: - - - -

*/
fn LDH_A_A8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr: u16 = 0xFF00 + get_imm8(cpu) as u16;
	let new_value = cpu.bus.borrow().read_byte(addr);

	set_debug_str(cpu, "a8", format!("0x{:X}", addr));

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LDH [a8], A
OPCODES: 0xE0
DESC: Loads the value stored in a into address 0xFF00+a8
FLAGS: - - - -

*/
fn LDH_A8_A(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let a_value = cpu.registers.get_8bit_reg(Register8Bit::A);
	let addr: u16 = 0xFF00 + get_imm8(cpu) as u16;

	set_debug_str(cpu, "a8", format!("0x{:X}", addr));

	cpu.bus.borrow_mut().write_byte(addr, a_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LDH A, [C]
OPCODES: 0xF2
DESC: Loads the value stored at 0xFF00+C into A
FLAGS: - - - -

*/
fn LDH_A_C(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr = 0xFF00 + cpu.get_8bit_reg(Register8Bit::C) as u16;
	let new_value = cpu.bus.borrow().read_byte(addr);

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LDH [C], A
OPCODES: 0xE2
DESC: Loads the value stored in A into address 0xFF00+C
FLAGS: - - - -

*/
fn LDH_C_A(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr = 0xFF00 + cpu.registers.get_8bit_reg(Register8Bit::C) as u16;
	let a_value = cpu.registers.get_8bit_reg(Register8Bit::A);

	cpu.bus.borrow_mut().write_byte(addr, a_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

// ! stack instructions

/*

MNEMONIC: PUSH r16
OPCODES: 0x(C-F)5
DESC: Pushes r16 onto the stack
FLAGS: - - - -

*/
fn PUSH_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let target = cpu.registers.get_16bit_reg(Register16Bit::from_r16stk((opcode >> 4) & 3));

	set_debug_str(cpu, "r16", format!("{:?}", Register16Bit::from_r16stk((opcode >> 4) & 3)));

	cpu.push16(target);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: POP r16
OPCODES: 0x(C-F)1
DESC: pops a value off the stack and stores it in r16
FLAGS: - - - -
FLAGS (POP AF): Z N H C

*/
fn POP_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let new_value = cpu.pop16();
	cpu.registers.set_16bit_reg(Register16Bit::from_r16stk((opcode >> 4) & 3), new_value);

	set_debug_str(cpu, "r16", format!("{:?}", Register16Bit::from_r16stk((opcode >> 4) & 3)));

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LD SP, HL
OPCODES: 0xF9
DESC: Loads the value stored in HL into SP
FLAGS: - - - -

*/
fn LD_SP_HL(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let hl_value = cpu.registers.get_16bit_reg(Register16Bit::HL);
	cpu.registers.set_16bit_reg(Register16Bit::SP, hl_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LD HL, SP + e8
OPCODES: 0xF8
DESC: Sets HL to SP + e8
FLAGS: 0 0 H C

*/
fn LD_HL_SP_E8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let sp_value = cpu.registers.get_16bit_reg(Register16Bit::SP);
	let offset: i8 = get_imm8(cpu) as i8;

	set_debug_str(cpu, "e8", format!("0x{:X}", offset));

	cpu.registers.set_16bit_reg(Register16Bit::HL, ((cpu.registers.get_16bit_reg(Register16Bit::SP) as i16).wrapping_add(offset as i16)) as u16);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::H, (sp_value & 0xF) + (offset as u16 & 0xF) > 0xF);
	cpu.registers.set_flag(Flag::C, (sp_value & 0xFF) + (offset as u16 & 0xFF) > 0xFF);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: ADD SP, e8
OPCODES: 0xE8
DESC: Offsets SP by e8
FLAGS: 0 0 H C

*/
fn ADD_SP_E8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
	let offset = get_imm8(cpu) as i8;

	set_debug_str(cpu, "e8", format!("0x{:X}", offset));

	let sp_value = cpu.registers.get_16bit_reg(Register16Bit::SP);
	cpu.registers.set_16bit_reg(Register16Bit::SP, ((sp_value as i16).wrapping_add(offset as i16)) as u16);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::H, (sp_value & 0xF) + (offset as u16 & 0xF) > 0xF);
	cpu.registers.set_flag(Flag::C, (sp_value & 0xFF) + (offset as u16 & 0xFF) > 0xFF);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: LD [a16], SP
OPCODES: 0x08
DESC: Loads a 16-bit immediate number into a 16-bit register (little endian)
FLAGS: - - - -

*/
fn LD_A16_SP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let addr = get_imm16(cpu);

	set_debug_str(cpu, "e8", format!("0x{:X}", addr));

	let sp = cpu.registers.get_16bit_reg(Register16Bit::SP);
	cpu.bus.borrow_mut().write_byte(addr, (sp & 0xFF) as u8);
	cpu.bus.borrow_mut().write_byte(addr + 1, (sp >> 8) as u8);

	cpu.pc = cpu.pc.wrapping_add(1);

}


// ! arithmetic instructions

/*

MNEMONIC: ADD A, r8/imm8 / ADC A, r8/imm8
OPCODES: 0x8(0-F)
DESC: Adds r8 to A
FLAGS: Z 0 H C

*/
fn ADD_ADC(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let mut carry: u8 = 0;

	if ((opcode & 0x8) != 0 || opcode == 0xCE) && cpu.registers.get_flag(Flag::C) {
		// add carry bit
		carry = 1;
	}

	let rhs_value = match opcode {
		0xC6 | 0xCE => {
			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "imm8", format!("0x{:X}", rhs));

			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	

	let new_value = cpu.add_8bit(a_value, rhs_value.wrapping_add(carry));

	cpu.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_flag(Flag::C, a_value as u16 + carry as u16 + rhs_value as u16 > 0xFF);
	cpu.registers.set_flag(Flag::H, (a_value & 0xF) + (carry & 0xF) + (rhs_value & 0xF) > 0xF);
	cpu.registers.set_flag(Flag::N, false);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: SUB A, r8/imm8 / SBC A, r8/imm8
OPCODES: 0x8(0-F)
DESC: Subtracts r8 from A
FLAGS: Z 1 H C

*/
fn SUB_SBC(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let mut carry: u8 = 0;

	if ((opcode & 8) != 0 || opcode == 0xDE) && cpu.registers.get_flag(Flag::C) {
		// subtract carry bit
		carry = 1;
	}

	let rhs_value = match opcode {
		0xD6 | 0xDE => {
			*cycles = 8;

			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "imm8", format!("0x{:X}", rhs));
			
			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	let mut new_value = cpu.sub_8bit(a_value, rhs_value);
	new_value = cpu.sub_8bit(new_value, carry);

	cpu.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_flag(Flag::C, (a_value as u16) < (carry as u16 + rhs_value as u16));
	cpu.registers.set_flag(Flag::H, (a_value & 0xF) < (rhs_value & 0xF) + carry);
	cpu.registers.set_flag(Flag::N, true);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: AND A, r8 / imm8
OPCODES: 0xA(0-7), 0xE6
DESC: Ands r8 with A
FLAGS: Z 0 1 0

*/
fn AND(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let rhs_value = match opcode {
		0xE6 => {
			*cycles = 8;

			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "n8", format!("0x{:X}", rhs));
			
			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	let new_value = a_value & rhs_value;
	cpu.set_8bit_reg(Register8Bit::A, new_value);

	cpu.set_8bit_reg(Register8Bit::F, 0);

	if new_value == 0 {
		cpu.registers.set_flag(Flag::Z, true);
	}

	cpu.registers.set_flag(Flag::H, true);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: XOR A, r8
OPCODES: 0xA(0-7)
DESC: Xors r8 with A
FLAGS: Z 0 0 0

*/
fn XOR(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let rhs_value = match opcode {
		0xEE => {
			*cycles = 8;

			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "imm8", format!("0x{:X}", rhs));
			
			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	let new_value = a_value ^ rhs_value;
	cpu.set_8bit_reg(Register8Bit::A, new_value);

	cpu.set_8bit_reg(Register8Bit::F, 0);

	if new_value == 0 {
		cpu.registers.set_flag(Flag::Z, true);
	}

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: OR A, r8
OPCODES: 0xA(0-7)
DESC: Ors r8 with A
FLAGS: Z 0 0 0

*/
fn OR(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let rhs_value = match opcode {
		0xF6 => {
			*cycles = 8;

			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "imm8", format!("0x{:X}", rhs));
			
			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	let new_value = a_value | rhs_value;
	cpu.set_8bit_reg(Register8Bit::A, new_value);

	cpu.set_8bit_reg(Register8Bit::F, 0);

	if new_value == 0 {
		cpu.registers.set_flag(Flag::Z, true);
	}

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: CP A, r8 / n8
OPCODES: 0x8(0-F)
DESC: Subtracts r8 / n8 from A, doesn't set A
FLAGS: Z 1 H C

*/
fn CP(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let a_value = cpu.get_8bit_reg(Register8Bit::A);
	let rhs_value = match opcode {
		0xFE => {
			*cycles = 8;

			let rhs = get_imm8(cpu);
			set_debug_str(cpu, "n8", format!("0x{:X}", rhs));
			
			rhs
		},
		_ => {
			let r8 = Register8Bit::from_r8(opcode & 7);
			if r8 == Register8Bit::HL { *cycles = 8; }

			set_debug_str(cpu, "r8", format!("{:?}", r8));

			cpu.get_8bit_reg(r8)
		},
	};

	cpu.sub_8bit(a_value, rhs_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: INC r16
OPCODES: 0x(0-3)3 
DESC: Increments a 16-bit register; doesnt set flags
FLAGS: - - - -

*/
fn INC_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	
	let reg = Register16Bit::from_r16(opcode >> 4);
	set_debug_str(cpu, "r16", format!("{:?}", reg));

	let old_flags = cpu.registers.get_8bit_reg(Register8Bit::F);

	let new_value = cpu.add_16bit(cpu.registers.get_16bit_reg(reg), 1);
	cpu.registers.set_16bit_reg(reg, new_value);

	// restore flags
	cpu.registers.set_8bit_reg(Register8Bit::F, old_flags);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: DEC r16
OPCODES: 0x(0-3)B 
DESC: Decrements a 16-bit registers; doesnt set flags
FLAGS: - - - -

*/
fn DEC_R16(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let reg = Register16Bit::from_r16(opcode >> 4);
	set_debug_str(cpu, "r16", format!("{:?}", reg));

	let old_flags = cpu.registers.get_8bit_reg(Register8Bit::F);

	let new_value = cpu.sub_16bit(cpu.registers.get_16bit_reg(reg), 1);
	cpu.registers.set_16bit_reg(reg, new_value);

	// restore flags
	cpu.registers.set_8bit_reg(Register8Bit::F, old_flags);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: INC r8
OPCODES: 0x(0-3)4, 0x(0-3)C 
DESC: Increments an 8-bit register (or [HL]); sets flags
FLAGS: Z 0 H -

*/
fn INC_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);
	set_debug_str(cpu, "r8", format!("{:?}", reg));

	let old_carry = cpu.registers.get_flag(Flag::C);

	let old_value = cpu.get_8bit_reg(reg);
	let new_value = cpu.add_8bit(old_value, 1);
	cpu.set_8bit_reg(reg, new_value);

	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc = cpu.pc.wrapping_add(1);

}

/*

MNEMONIC: DEC r8
OPCODES: 0x(0-3)5, 0x(0-3)D
DESC: Decrements an 8-bit register
FLAGS: Z 1 H -

*/
fn DEC_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let reg = Register8Bit::from_r8(opcode >> 3);
	set_debug_str(cpu, "r8", format!("{:?}", reg));

	let old_carry = cpu.registers.get_flag(Flag::C);

	let old_value = cpu.get_8bit_reg(reg);
	let new_value = cpu.sub_8bit(old_value, 1);
	cpu.set_8bit_reg(reg, new_value);

	cpu.registers.set_flag(Flag::N, true);
	cpu.registers.set_flag(Flag::C, old_carry);

	cpu.pc = cpu.pc.wrapping_add(1);

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

	set_debug_str(cpu, "r16", format!("{:?}", rhs));

	let old_z = cpu.registers.get_flag(Flag::Z);

	let new_hl = cpu.add_16bit(lhs, rhs);
	cpu.registers.set_16bit_reg(Register16Bit::HL, new_hl);

	cpu.registers.set_flag(Flag::Z, old_z);

	cpu.pc = cpu.pc.wrapping_add(1);
}

// ! 8-bit shift, rotate and bit instructions

/*

MNEMONIC: RLCA, RRCA
OPCODES: 0x07, 0x0F
DESC: Rotates the A register to the left / right. The carry bit is set to the shifted out bit
FLAGS: 0 0 0 C

*/
fn RLCA_RRCA(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);

	let old_a = cpu.registers.get_8bit_reg(Register8Bit::A);

	if opcode >> 3 == 1 {
		// RRCA
		new_value = cpu.registers.get_8bit_reg(Register8Bit::A).rotate_right(1);
		carry = old_a & 1 == 1;
	} else {
		// RLCA
		new_value = cpu.registers.get_8bit_reg(Register8Bit::A).rotate_left(1);
		carry = (old_a & 0x80) >> 7 == 1;
	}

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: RLA, RRA
OPCODES: 0x17, 0x1F
DESC: Rotates the A register to the left / right. Wraps around to the carry bit, then carry bit is set to the shifted out bit.
FLAGS: 0 0 0 C

*/
fn RLA_RRA(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);
	let is_rla = opcode == 0x17;

	let old_a = cpu.registers.get_8bit_reg(Register8Bit::A);

	if is_rla {
		// RLA
		new_value = cpu.registers.get_8bit_reg(Register8Bit::A) << 1;
		carry = (old_a & 0x80) >> 7 == 1;
	} else {
		// RRA
		new_value = cpu.registers.get_8bit_reg(Register8Bit::A) >> 1;
		carry = old_a & 1 == 1;
	}

	let old_carry = match is_rla {
		true => cpu.registers.get_flag(Flag::C) as u8,
		false => (cpu.registers.get_flag(Flag::C) as u8) << 7
	};

	cpu.registers.set_8bit_reg(Register8Bit::A, new_value | old_carry);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*
?##################################################
?##########	  Prefixed Instructions	 	###########
?##################################################
*/

/*

MNEMONIC: RLC r8 / RRC r8
OPCODES: 0x0(0-7) / 0x0(8-F)
DESC: Rotates left / right into the carry flag
FLAGS: Z 0 0 C

*/
fn RLC_RRC_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);

	let r8 = Register8Bit::from_r8(opcode & 0x7);

	set_debug_str(cpu, "r8", format!("{:?}", r8));

	let old_value = cpu.get_8bit_reg(r8);

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}
	
	// ? when rotating values the value shifted out should wrap back around (rotating != shifting)

	if opcode >> 3 == 1 {
		// RRC
		new_value = old_value.rotate_right(1);
		carry = (old_value & 0x01) != 0;
	} else {
		//RLC
		new_value = old_value.rotate_left(1);
		carry = (old_value & 0x80) != 0;
	}

	cpu.set_8bit_reg(r8, new_value);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);
	cpu.registers.set_flag(Flag::Z, new_value == 0);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: RL r8 / RR r8
OPCODES: 0x1(0-7), 0x1(8-F)
DESC: Rotates r8 to the left / right. Wraps around to the carry bit, then carry bit is set to the shifted out bit.
FLAGS: Z 0 0 C

*/
fn RL_RR_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let r8 = Register8Bit::from_r8(opcode & 0x7);

	set_debug_str(cpu, "r8", format!("{:?}", r8));

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	let old_value = cpu.get_8bit_reg(r8);

	let (new_value, carry): (u8, bool);
	let is_rl = (opcode >> 3) & 1 == 0;

	if is_rl {
		new_value = old_value << 1;
		carry = (old_value & 0x80) >> 7 == 1;
	} else {
		new_value = old_value >> 1;
		carry = old_value & 1 == 1;
	}

	let old_carry = match is_rl {
		true => cpu.registers.get_flag(Flag::C) as u8,
		false => (cpu.registers.get_flag(Flag::C) as u8) << 7
	};

	cpu.set_8bit_reg(r8, new_value | old_carry);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);
	cpu.registers.set_flag(Flag::Z, (new_value | old_carry) == 0);

	cpu.pc = cpu.pc.wrapping_add(1);

}

/*

MNEMONIC: SLA r8 / SRA r8
OPCODES: 0x2(0-7) / 0x2(8-F)
DESC: Shifts left / right into the carry flag
FLAGS: Z 0 0 C

*/
fn SLA_SRA_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let (new_value, carry): (u8, bool);

	let r8 = Register8Bit::from_r8(opcode & 0x7);

	set_debug_str(cpu, "r8", format!("{:?}", r8));

	let old_value = cpu.get_8bit_reg(r8);

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	let is_sla = (opcode >> 3) & 0x01 == 0;

	if is_sla {
		// SLA
		(new_value, _) = old_value.overflowing_shl(1);
		carry = (old_value & 0x80) >> 7 == 1;
	} else {
		//SRA
		(new_value, _) = old_value.overflowing_shr(1);
		carry = old_value & 1 == 1;
	}

	//? Arithmetic shifts preserve the sign bit (bit 7), doesnt apply to left shift (https://open4tech.com/wp-content/uploads/2016/11/Arithmetic_Shift.jpg)
	if !is_sla {
		cpu.set_8bit_reg(r8, new_value | (old_value & 0x80));
	} else {
		cpu.set_8bit_reg(r8, new_value);
	}

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);
	cpu.registers.set_flag(Flag::Z, new_value == 0);

	cpu.pc = cpu.pc.wrapping_add(1);

}

/*

MNEMONIC: SWAP r8
OPCODES: 0x3(0-7)
DESC: Swaps the lower four bits of r8 with the upper four
FLAGS: Z 0 0 0

*/
fn SWAP_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let r8 = Register8Bit::from_r8(opcode & 0x7);
	let old_value = cpu.get_8bit_reg(r8);

	set_debug_str(cpu, "r8", format!("{:?}", r8));

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	let new_value = ((old_value & 0xF) << 0x4) | (old_value & 0xF0) >> 0x4;

	cpu.set_8bit_reg(r8, new_value);

	cpu.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::Z, new_value == 0);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: SRL r8
OPCODES: 0x3(8-F)
DESC: Logical shift r8 to the right
FLAGS: Z 0 0 C

*/
fn SRL_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {
	let (new_value, carry): (u8, bool);

	let r8 = Register8Bit::from_r8(opcode & 0x7);
	set_debug_str(cpu, "r8", format!("{:?}", r8));

	let old_value = cpu.get_8bit_reg(r8);

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	(new_value, _) = old_value.overflowing_shr(1);
	carry = old_value & 0x01 == 1;

	cpu.set_8bit_reg(r8, new_value);

	cpu.registers.set_8bit_reg(Register8Bit::F, 0);
	cpu.registers.set_flag(Flag::C, carry);
	cpu.registers.set_flag(Flag::Z, new_value == 0);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: BIT u3, r8
OPCODES: 0x(4-7)(0-F)
DESC: Tests bit u3 in r8 (bit == 0).
FLAGS: Z 0 1 -

*/
fn BIT_U3_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let r8 = Register8Bit::from_r8(opcode & 0x7);
	let u3 = (opcode >> 3) & 0x7;

	set_debug_str(cpu, "r8", format!("{:?}", r8));
	set_debug_str(cpu, "u3", format!("{}", u3));

	if r8 == Register8Bit::HL {
		*cycles = 12;
	}

	let test_bit = (cpu.get_8bit_reg(r8)) & (1 << u3);

	cpu.registers.set_flag(Flag::Z, test_bit == 0);
	cpu.registers.set_flag(Flag::N, false);
	cpu.registers.set_flag(Flag::H, true);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: RES u3, r8
OPCODES: 0x(8-B)(0-F)
DESC: Sets bit u3 in r8 to 0
FLAGS: - - - -

*/
fn RES_U3_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let r8 = Register8Bit::from_r8(opcode & 0x7);
	let u3 = (opcode >> 3) & 0x7;

	set_debug_str(cpu, "r8", format!("{:?}", r8));
	set_debug_str(cpu, "u3", format!("{}", u3));

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	let new_value = cpu.get_8bit_reg(r8) & !(1 << u3);

	cpu.set_8bit_reg(r8, new_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}

/*

MNEMONIC: SET u3, r8
OPCODES: 0x(C-F)(0-F)
DESC: Sets bit u3 in r8 to 1
FLAGS: - - - -

*/
fn SET_U3_R8(cpu: &mut CPU, opcode: u8, cycles: &mut u16) {

	let r8 = Register8Bit::from_r8(opcode & 0x7);
	let u3 = (opcode >> 3) & 0x7;

	set_debug_str(cpu, "r8", format!("{:?}", r8));
	set_debug_str(cpu, "u3", format!("{}", u3));

	if r8 == Register8Bit::HL {
		*cycles = 16;
	}

	let new_value = cpu.get_8bit_reg(r8) | (1 << u3);

	cpu.set_8bit_reg(r8, new_value);

	cpu.pc = cpu.pc.wrapping_add(1);
}
