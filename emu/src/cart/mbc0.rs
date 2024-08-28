use super::{Cart, MBC};

pub struct MBC0 {
	rom: Vec<u8>
}

impl MBC0 {

	pub fn new(rom: Vec<u8>) -> Self {
		
		Self {
			rom: rom
		}

	}

}

impl MBC for MBC0 {

	fn read(&self, addr: u16) -> u8 {
		self.rom[addr as usize]
	}

	fn write(&mut self, addr: u16, write: u8) {}

}