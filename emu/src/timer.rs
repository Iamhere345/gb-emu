use crate::interrupt::{Interrupt, InterruptFlag};

use std::cell::RefCell;
use std::rc::Rc;

struct Clock {
	carry: u64,
	period: u64
}

impl Clock {
	
	fn new(period: u64) -> Self {
		Self {
			carry: 0,
			period: period
		}
	}

	fn tick(&mut self, cycles: u64) -> u64 {

		self.carry += cycles;
		let inc = self.carry / self.period;
		self.carry %= self.period;

		inc

	}

}

pub struct Timer {
	intf: Rc<RefCell<Interrupt>>,

	div: u8,
	div_clock: Clock,

	tima: u8,
	tima_clock: Clock,

	tma: u8,
	tac: u8
}

impl Timer {

	pub fn new(intf: Rc<RefCell<Interrupt>>) -> Self {

		Self {
			intf: intf,

			div: 0,
			div_clock: Clock::new(256),

			tima: 0,
			tima_clock: Clock::new(1024),

			tma: 0,
			tac: 0
		}

	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			0xFF04 => self.div,
			0xFF05 => self.tima,
			0xFF06 => self.tma,
			0xFF07 => self.tac,
			_ => panic!("invalid address")
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			0xFF04 => self.div = 0,
			0xFF05 => self.tima = write,
			0xFF06 => self.tma = write,
			0xFF07 => {
				if (self.tac & 0x3) != (write & 0x3) {
					self.tima_clock.carry = 0;
					self.tima_clock.period = match write & 0x3 {
						0b00 => 1024,	// 4096Hz, update every 1024 cycles 
						0b01 => 16,		// 262144Hz, update every 16 cycles
						0b10 => 64,		// 65536Hz, update every 64 cycles
						0b11 => 256,	// 16384Hz, update every 256 cycles
						_ => panic!("invalid speed")
					};

					self.tima = self.tma;
				}

				self.tac = write;
			}
			_ => panic!("invalid address")
		}
	}

	pub fn tick(&mut self, cycles: u64) {

		self.div = self.div.wrapping_add(self.div_clock.tick(cycles) as u8);

		// if TIMA is enabled
		if (self.tac & 0x4) != 0 {

			let inc = self.tima_clock.tick(cycles);

			for _ in 0..inc {

				self.tima = self.tima.wrapping_add(1);

				if self.tima == 0 {
					self.tima = self.tma;

					// raise interrupt
					self.intf.borrow_mut().raise(InterruptFlag::Timer);
				}
			}
		}

	}

}