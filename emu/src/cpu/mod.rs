pub mod registers;
mod instructions;

use crate::bus::*;
use crate::interrupt::*;
use self::registers::*;
use self::instructions::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct CPU {
	pub registers: Registers,
	pub bus: Rc<RefCell<Bus>>,
	pub pc: u16,
	pub ime: bool,			// interrupt master enable
	pub ei: u8,				// ei instruction executed; wait another cycle before enabling ime
	pub halted: bool,		// used with the HALT instruction
	pub instr_cycles: u64,	// the amount of cycles the last instruction took
	pub last_instruction: String,
}

#[allow(dead_code)]
impl CPU {

	pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
		CPU {
			registers: Registers::new(),
			bus: bus,
			pc: 0x100,
			ime: false,
			ei: 0,
			halted: false,
			instr_cycles: 0,
			last_instruction: String::new(),
		}
	}

	pub fn cycle(&mut self) -> u64 {

		self.ei = match self.ei {
			1 => 2,
			2 => {
				self.ime = true;
				0
			},
			_ => 0
		};

		let int_cycles = self.interrupt();

		if int_cycles != 0 {
			int_cycles
		} else if self.halted {
			4	// effectively a NOP
		} else {
			self.exec()
		}
	}

	pub fn exec(&mut self) -> u64 {
		let mut byte: u8 = self.bus.borrow().read_byte(self.pc);
		let prefixed: bool;

		if byte == 0xCB {

			//println!("[0x{:x}] prefixed 0xCB", self.pc);

			// prefixed instructions
			prefixed = true;

			self.pc = self.pc.wrapping_add(1);
			byte = self.bus.borrow().read_byte(self.pc);

		} else {
			prefixed = false;
		}

		let mut executed: bool = false;
		let mut instr_cycles: u64 = 4;

		'instruction_execute:
		for instruction in if prefixed { PREFIXED_INSTRUCTIONS.iter() } else { INSTRUCTIONS.iter() } {
			for opcode in instruction.opcodes.iter() {
				if byte == *opcode {

					//let last_pc = self.pc;

					self.last_instruction = instruction.mnemonic.to_string();

					let mut cycles = instruction.cycles;
					
					(instruction.exec)(self, byte, &mut cycles);
					
					//println!("[0x{:x}][0x{:x}] {}", last_pc, byte, self.last_instruction);

					instr_cycles = cycles as u64;
					executed = true;

					break 'instruction_execute;
				}
			}
		}

		if !executed {
			//println!("[0x{:x}] Undefined opcode: 0x{:x}", self.pc, byte);
		}

		instr_cycles

	}
	
	pub fn interrupt(&mut self) -> u64 {
		// check for interrupts
		if self.ime || self.halted {
			let if_flags = self.bus.borrow().read_register(MemRegister::IF);
			let ie_flags = self.bus.borrow().read_register(MemRegister::IE);

			for i in 0..4 {

				if ((if_flags >> i) & 1) == 1 && ((ie_flags >> i) & 1) == 1 {
					let flag: InterruptFlag = InterruptFlag::from_u8(((if_flags >> i) & 1) << i);
					let source = InterruptSource::from_flag(flag);

					// handle interrupt
					if self.halted { 
						self.halted = false;
			
						return 0;
					}
					
					// clear the bit in IF
					let new_if = self.bus.borrow().read_register(MemRegister::IF) & !(flag as u8);
					self.bus.borrow_mut().write_register(MemRegister::IF, new_if);
			
					self.ime = false;
			
					self.push16(self.pc);
					self.pc = source as u16;

					return 25; // 5 M-cycles
				}

			}

		}

		return 0;
	}
	
	pub fn push16(&mut self, to_push: u16) {
		let mut target_addr = self.dec_sp();
		self.bus.borrow_mut().write_byte(target_addr, (to_push >> 8) as u8); // high byte

		target_addr = self.dec_sp();
		self.bus.borrow_mut().write_byte(target_addr, (to_push & 0xFF) as u8); // low byte
	}

	pub fn pop16(&mut self) -> u16 {
		let low_byte = self.bus.borrow().read_byte(self.registers.get_16bit_reg(Register16Bit::SP));
		let sp = self.inc_sp();
		let hi_byte = self.bus.borrow().read_byte(sp);
		self.inc_sp();

		(hi_byte as u16) << 8 | low_byte as u16
	}

	pub fn get_deref_hl(&self) -> u8 {
		self.bus.borrow().read_byte(self.registers.get_16bit_reg(Register16Bit::HL))
	}

	pub fn set_deref_hl(&mut self, write: u8) {
		self.bus.borrow_mut().write_byte(self.registers.get_16bit_reg(Register16Bit::HL), write);
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
		self.registers.set_flag(Flag::H, (lhs & 0xF).overflowing_add(rhs & 0xF).0 > 0xF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	// adds rhs to reg, update flags
	pub fn add_16bit(&mut self, lhs: u16, rhs: u16) -> u16 {
		let (new_value, did_overflow) = lhs.overflowing_add(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, false);
		// check if there was an overflow from the 11th bit (0b_0000_1000_0000_0000)
		self.registers.set_flag(Flag::H, (lhs & 0xFFF).overflowing_add(rhs & 0xFFF).0 > 0xFFF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn sub_8bit(&mut self, lhs: u8, rhs: u8) -> u8 {
		let (new_value, did_overflow) = lhs.overflowing_sub(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, true);
		self.registers.set_flag(Flag::H, (lhs & 0xF).overflowing_sub(rhs & 0xF).0 > 0xF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn sub_16bit(&mut self, lhs: u16, rhs: u16) -> u16 {
		let (new_value, did_overflow) = lhs.overflowing_sub(rhs);

		self.registers.set_flag(Flag::C, did_overflow);
		self.registers.set_flag(Flag::N, true);
		self.registers.set_flag(Flag::H, (lhs & 0xFFF).overflowing_sub(rhs & 0xFFF).0 < 0xFFF);
		self.registers.set_flag(Flag::Z, new_value == 0);

		new_value
	}

	pub fn inc_pc(&mut self) -> u16 {
		self.pc += 1;

		self.pc
	}

	pub fn inc_sp(&mut self) -> u16 {
		let old_value = self.registers.get_16bit_reg(Register16Bit::SP);
		self.registers.set_16bit_reg(Register16Bit::SP, old_value.overflowing_add(1).0);

		old_value.overflowing_add(1).0
	}

	pub fn dec_sp(&mut self) -> u16 {
		let old_value = self.registers.get_16bit_reg(Register16Bit::SP);
		self.registers.set_16bit_reg(Register16Bit::SP, old_value.overflowing_sub(1).0);

		old_value.overflowing_sub(1).0
	}
}