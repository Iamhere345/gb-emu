mod mbc0;
mod mbc1;
mod mbc3;
mod mbc5;

pub trait MBC {
	fn read(&self, addr: u16) -> u8;
	fn write(&mut self, addr: u16, write: u8);

	fn is_battery_backed(&self) -> bool;
	fn load_sram(&mut self, sram: Vec<u8>);
	fn dump_sram(&self) -> Vec<u8>;
}

pub fn create_cart(rom: Vec<u8>) -> Box<dyn MBC> {

	let ram_size = match rom[0x149] {
		2 			=> 8 * 1024,
		3 			=> 32 * 1024,
		4 			=> 128 * 1024,
		5			=> 64 * 1024,
		0 | 1 | _ 	=> 0,
	};

	match rom[0x147] {
		0x0 	=> Box::new(mbc0::MBC0::new(rom)),
		0x1 | 0x2 | 0x3 => Box::new(mbc1::MBC1::new(rom, ram_size)),
		0xF | 0x10 | 0x11 | 0x12 | 0x13 => Box::new(mbc3::MBC3::new(rom, ram_size)),
		0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => Box::new(mbc5::MBC5::new(rom, ram_size)),

		_ => panic!("unimplenented cart type 0x{:X}", rom[0x147])
	}

}