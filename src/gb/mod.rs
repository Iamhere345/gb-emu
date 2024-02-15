use cpu::CPU;
use bus::Bus;

use std::cell::RefCell;
use std::rc::Rc;

pub mod cpu;
pub mod bus;

pub struct Gameboy {
	cpu: CPU,
	bus: Rc<RefCell<Bus>>,
	pub cycles: u64,
}

impl Gameboy {

	
	pub fn new() -> Gameboy {

		let bus = Rc::new(RefCell::new(Bus::new()));

		Gameboy {
			bus: Rc::clone(&bus),
			cpu: CPU::new(Rc::clone(&bus)),
			cycles: 0
		}

	}

}
