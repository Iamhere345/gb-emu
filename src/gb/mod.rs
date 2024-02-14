use cpu::CPU;
use bus::Bus;

use std::cell::RefCell;

pub mod cpu;
pub mod bus;

pub struct Gameboy<'a> {
	cpu: CPU<'a>,
	cpu2: CPU<'a>,
	bus: RefCell<Bus>,
	pub cycles: u64,
}

impl<'a> Gameboy<'a> {

	
	pub fn new() -> Gameboy<'a> {

		// TODO use RefCell<T> for pointers to the bus?
		// AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
		// i fucking hate lifetimes
		let bus = RefCell::new(Bus::new());

		Gameboy {
			bus: bus,
			cpu: CPU::new(&bus),
			cpu2: CPU::new(&bus),
			cycles: 0
		}

	}

}
