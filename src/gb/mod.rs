use cpu::*;
use cpu::registers::*;
use bus::Bus;

use std::cell::RefCell;
use std::rc::Rc;

use std::io::{prelude::*, BufWriter};
use std::fs::File;

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

	pub fn init(&mut self, cart: &'static [u8]) {

		for (i, byte) in cart.iter().enumerate() {
			self.bus.borrow_mut().write_byte(i.try_into().unwrap(), *byte)
		}

		let log = File::create("emu.log").expect("unable to open log file");
		let mut log_writer = BufWriter::new(&log);

		self.cpu.pc = 0xFF;

		// temp
		loop {
			let result = self.cpu.cycle();
	
			let a = self.cpu.registers.get_8bit_reg(Register8Bit::A);
			let f = self.cpu.registers.get_8bit_reg(Register8Bit::F);
			let b = self.cpu.registers.get_8bit_reg(Register8Bit::B);
			let c = self.cpu.registers.get_8bit_reg(Register8Bit::C);
			let d = self.cpu.registers.get_8bit_reg(Register8Bit::D);
			let e = self.cpu.registers.get_8bit_reg(Register8Bit::E);
			let h = self.cpu.registers.get_8bit_reg(Register8Bit::H);
			let l = self.cpu.registers.get_8bit_reg(Register8Bit::L);
			let sp = self.cpu.registers.get_16bit_reg(Register16Bit::SP);
			let pc = self.cpu.pc;
			let pc0 = self.bus.borrow().read_byte(self.cpu.pc);
			let pc1 = self.bus.borrow().read_byte(self.cpu.pc + 1);
			let pc2 = self.bus.borrow().read_byte(self.cpu.pc + 2);
			let pc3 = self.bus.borrow().read_byte(self.cpu.pc + 3);
	
			write!(&mut log_writer, "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n", a,f,b,c,d,e,h,l,sp,pc,pc0,pc1,pc2,pc3).expect("E");
	
			if result == true {
				break;
			}
		}

	}

}
