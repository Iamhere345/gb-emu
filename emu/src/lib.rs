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
	pub cpu: CPU,
	pub bus: Rc<RefCell<Bus>>,
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

		self.cpu.pc = 0x100;

		// temp
		/*
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
		*/

	}

	pub fn tick(&mut self, /*log: &mut BufWriter<File>*/) {

		

		self.cycles += 4;

		if self.cpu.wait_cycles == 0 {

			/*
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
			*/

			//println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n", 
			//a,f,b,c,d,e,h,l,sp,pc,pc0,pc1,pc2,pc3);

			//log.write_all(format!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n", 
			//a,f,b,c,d,e,h,l,sp,pc,pc0,pc1,pc2,pc3).as_bytes()).expect("unable to write to log file");

			//log.flush().expect("unable to flush buffer to log file");
			
			self.cpu.cycle();
			
			if self.bus.borrow().read_byte(0xFF02) == 0x81 {

				let serial_char: char = self.bus.borrow().read_byte(0xFF01) as char;

				print!("{}", serial_char);

				self.bus.borrow_mut().write_byte(0xFF02, 0);
			}


			/*
			write!(log,
				"A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n", 
				a,f,b,c,d,e,h,l,sp,pc,pc0,pc1,pc2,pc3).expect("unable to write to log file");
			*/

		} else {
			//println!("tick.");
			self.cpu.wait_cycles = 0;
		}

	}

}
