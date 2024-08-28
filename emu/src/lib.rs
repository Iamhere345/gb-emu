use cpu::*;
use bus::Bus;
use cpu::registers::{Register8Bit, Register16Bit};

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

pub mod cpu;
pub mod bus;
pub mod interrupt;
pub mod timer;
pub mod ppu;
pub mod joypad;
pub mod cart;

const CYCLES_PER_SCANLINE: u64 = 69905;

pub struct Gameboy {
	pub bus: Rc<RefCell<Bus>>,
	pub cpu: CPU,
	pub cycles: u64,	// clock cycles in T-states
}

impl Gameboy {

	
	pub fn new(cart: Vec<u8>) -> Gameboy {

		let bus = Rc::new(RefCell::new(Bus::new(cart)));

		Gameboy {
			bus: Rc::clone(&bus),
			cpu: CPU::new(Rc::clone(&bus)),
			cycles: 0
		}

	}

	pub fn tick(&mut self) -> u64 {

		let instr_cycles = self.cpu.cycle();

		self.bus.borrow_mut().timer.tick(instr_cycles);
		self.bus.borrow_mut().ppu.tick(instr_cycles);

		if self.bus.borrow().read_byte(0xFF02) == 0x81 {

			let serial_char: char = self.bus.borrow().read_byte(0xFF01) as char;

			//print!("{}", serial_char);

			self.bus.borrow_mut().write_byte(0xFF02, 0);
		}

		instr_cycles

	}

	pub fn run_frame(&mut self) {

		loop {

			let cycles = self.tick();

			self.cycles += cycles;

			if self.cycles >= CYCLES_PER_SCANLINE {
				self.cycles -= CYCLES_PER_SCANLINE;
				break;
			}

		}

	}

	pub fn run_scanline(&mut self) {

		let current_ly = self.bus.borrow().read_register(bus::MemRegister::LY);

		while self.bus.borrow().read_register(bus::MemRegister::LY) != current_ly.wrapping_add(1) {
			self.cycles += self.tick();
		}

	}

}
