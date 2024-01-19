use crate::instructions;
use crate::registers::*;
use crate::bus::*;
use crate::instructions::*;

pub struct CPU<'a> {
	pub registers: Registers,
	pub pc: u16,
	pub bus: &'a mut Bus,
}

#[allow(dead_code)]
impl<'a> CPU<'a> {

	pub fn new(bus: &'a mut Bus) -> Self {
		CPU {
			registers: Registers::new(),
			pc: 0,
			bus: bus,
		}
	}

	pub fn cycle(&mut self, instructions: &Vec<Instruction>, prefixed_instructions: &Vec<Instruction>) {

		// detect prefix byte CB

			// decode opcode
			// based on opcode, decode displacement and immediate data bytes
		
		// detect prefix bytes DD/FD, CB
			// displacement byte
			// opcode

		// TODO check if the amount of cycles for an instruction includes fetching the byte after a prefix and immediates (i think it does)

		let mut byte: u8 = self.bus.read_byte(self.pc);
		let instruction_set: &Vec<Instruction>;

		if byte == 0xCB {

			println!("[0x{:x}] prefixed 0xCB", self.pc);

			// prefixed instructions
			instruction_set = &prefixed_instructions;

			self.pc += 1;
			byte = self.bus.read_byte(self.pc);

			// TODO look into DD FD prefixes (i think they might be z80 only but they're still specified in some documents)
		} else {
			instruction_set = &instructions;
		}

		for instruction in instruction_set.iter() {
			for opcode in instruction.opcodes.iter() {
				if byte == *opcode {

					println!("[0x{:x}] {}", self.pc, instruction.mnemonic);

					(instruction.exec)(self, byte, instruction.cycles);

					// TODO wait clock_speed * cycles after the instruction has been executed
					return;
				}
			}
		}

		println!("[0x{:x}] Undefined opcode: 0x{:x}", self.pc, byte);
		

	}
	
	pub fn get_deref_hl(&self) -> u8 {
		self.bus.read_byte(self.registers.get_16bit_reg(Register16Bit::HL))
	}

	pub fn set_deref_hl(&mut self, write: u8) {
		self.bus.write_byte(self.registers.get_16bit_reg(Register16Bit::HL), write);
	}

	/*
		8bit register wrappers that include [HL]
	*/

	pub fn get_8bit_reg(&mut self, reg: Register8Bit) -> u8 {
		if reg == Register8Bit::HL {
			self.get_deref_hl()
		} else {
			self.registers.get_8bit_reg(reg)
		}
	}

	pub fn set_8bit_reg(&mut self, reg: Register8Bit, write: u8) {
		if reg == Register8Bit::HL {
			self.set_deref_hl(write);
		} else {
			self.registers.set_8bit_reg(reg, write);
		}
	}

	// adds rhs to reg, update flags
	pub fn add_8bit(&mut self, lhs: u8, rhs: u8) -> u8 {
		let (new_value, did_overflow) = lhs.overflowing_add(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, false);
		self.registers.set_flag(Flag::H, (new_value & 0xF) + (rhs & 0xF) > 0xF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	// adds rhs to reg, update flags
	pub fn add_16bit(&mut self, lhs: u16, rhs: u16) -> u16 {
		let (new_value, did_overflow) = lhs.overflowing_add(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, false);
		// check if there was an overflow from the 11th bit (0b_0000_1000_0000_0000)
		self.registers.set_flag(Flag::H, (new_value & 0x800) + (rhs & 0x800) > 0x800);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn sub_8bit(&mut self, lhs: u8, rhs: u8) -> u8 {
		let (new_value, did_overflow) = lhs.overflowing_sub(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, true);
		self.registers.set_flag(Flag::H, (new_value & 0xF) - (rhs & 0xF) > 0xF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn sub_16bit(&mut self, lhs: u16, rhs: u16) -> u16 {
		let (new_value, did_overflow) = lhs.overflowing_sub(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, true);
		self.registers.set_flag(Flag::H, (new_value & 0x800) - (rhs & 0x800) > 0x800);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn inc_pc(&mut self) -> u16 {
		self.pc += 1;

		self.pc
	}
}