use cpu::*;
use bus::Bus;

use std::cell::RefCell;
use std::rc::Rc;

pub mod cpu;
pub mod bus;
pub mod interrupt;
pub mod timer;
pub mod ppu;
pub mod joypad;
pub mod cart;
pub mod apu;

const CYCLES_PER_FRAME: u64 = 69905;

pub struct Gameboy {
	pub bus: Rc<RefCell<Bus>>,
	pub cpu: CPU,
	pub cycles: u64,	// clock cycles in T-states
}

impl Gameboy {

	pub fn new(cart: Vec<u8>, audio_callback: Box<dyn Fn(&[f32])>) -> Gameboy {

		let bus = Rc::new(RefCell::new(Bus::new(cart, audio_callback)));

		Gameboy {
			bus: Rc::clone(&bus),
			cpu: CPU::new(Rc::clone(&bus)),
			cycles: 0
		}

	}

	pub fn tick(&mut self) -> bool {

		let instr_cycles = self.cpu.cycle();

		self.bus.borrow_mut().timer.tick(instr_cycles);
		let buffer_full = self.bus.borrow_mut().apu.tick(instr_cycles);
		self.bus.borrow_mut().ppu.tick(instr_cycles);

		buffer_full
	}

	pub fn run_scanline(&mut self) {

		let current_ly = self.bus.borrow().read_register(bus::MemRegister::LY);

		while self.bus.borrow().read_register(bus::MemRegister::LY) != current_ly.wrapping_add(1) {
			self.tick();
		}

	}

	pub fn load_bootrom(&mut self, bootrom: Vec<u8>, bootrom_enable: bool) {
		self.bus.borrow_mut().bootrom = bootrom;
		self.bus.borrow_mut().bootrom_loaded = bootrom_enable;

		if bootrom_enable { self.cpu.pc = 0 }
	}

}
