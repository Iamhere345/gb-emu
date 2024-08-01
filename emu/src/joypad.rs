use crate::interrupt::{Interrupt, InterruptFlag};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum GBInput {
	DPadUp,
	DPadDown,
	DPadLeft,
	DPadRight,
	BtnA,
	BtnB,
	BtnSelect,
	BtnStart,
}

impl GBInput {

	pub fn into_mask(&self) -> u8 {
		match self {
			&GBInput::BtnA 		=> 1 << 0,
			&GBInput::BtnB 		=> 1 << 1,
			&GBInput::BtnSelect => 1 << 2,
			&GBInput::BtnStart 	=> 1 << 3,
			
			&GBInput::DPadRight	=> 1 << 0,
			&GBInput::DPadLeft	=> 1 << 1,
			&GBInput::DPadUp	=> 1 << 2,
			&GBInput::DPadDown 	=> 1 << 3,
		}
	}

	pub fn is_dpad(&self) -> bool {
		match self {
			&GBInput::BtnA 		=> false,
			&GBInput::BtnB 		=> false,
			&GBInput::BtnSelect => false,
			&GBInput::BtnStart 	=> false,
			
			&GBInput::DPadRight	=> true,
			&GBInput::DPadLeft	=> true,
			&GBInput::DPadUp	=> true,
			&GBInput::DPadDown 	=> true,
		}
	}

}

pub struct Joypad {
	intf: Rc<RefCell<Interrupt>>,

	dpad_state: u8,
	btn_state: u8,
	select: u8,
}

impl Joypad {

	pub fn new(intf: Rc<RefCell<Interrupt>>) -> Self {
		Self {
			intf: intf,

			dpad_state: 0xF,
			btn_state: 0xF,
			select: 0 
		}
	}

	pub fn read(&self) -> u8 {

		println!("read joyp select: 0b{:b} dpad: 0b{:b} btn 0b{:b}", self.select, self.dpad_state, self.btn_state);

		match self.select {
			0b01 => (self.select << 4) | self.dpad_state,
			0b10 => (self.select << 4) | self.btn_state,
			0b11 => (self.select << 4) | 0xF,
			_ 	 => panic!("invalid joyp select value"),
		}
	}

	pub fn write(&mut self, write: u8) {

		//println!("write 0b{:b}", write);

		self.select = (write >> 4) & 0b11;

	}

	pub fn btn_down(&mut self, input: GBInput) {

		//println!("btn down {:?}", input);

		match input.is_dpad() {
			true 	=> self.dpad_state &= !input.into_mask() as u8,
			false 	=> self.btn_state &= !input.into_mask() as u8,
		};

		self.intf.borrow_mut().raise(InterruptFlag::Joypad);

	}

	pub fn btn_up(&mut self, input: GBInput) {

		//println!("btn up {:?}", input);

		match input.is_dpad() {
			true 	=> self.dpad_state |= input.into_mask() as u8,
			false 	=> self.btn_state |= input.into_mask() as u8,
		};

	}

}