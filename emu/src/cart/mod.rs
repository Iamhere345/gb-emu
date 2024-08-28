mod mbc0;

pub trait MBC {
	fn read(&self, addr: u16) -> u8;
	fn write(&mut self, addr: u16, write: u8);
}

struct CartInfo {
	cart_type: u8,
	
}

struct Cart {
	rom: Vec<u8>,
	ram: Option<Vec<u8>>,

	rom_bank: u8,
	ram_bank: u8,
	
	ram_enabled: bool
}

impl Cart {

	pub fn new(rom: Vec<u8>, has_ram: bool, ram_size: usize) -> Self {
		
		let ram = if has_ram { Some(vec![0; ram_size]) } else { None };

		Self {
			rom: rom,
			ram: ram,
			
			rom_bank: 1,
			ram_bank: 0,

			ram_enabled: false
		}

	}
	
	pub fn read(&self, addr: u16) -> u8 {

		match addr {
			0		..= 0x3FFF	=> self.rom[addr as usize],
			0x4000	..= 0x7FFF	=> self.rom[(addr as usize - 0x4000) + (0x4000 * self.rom_bank as usize)],
			0xA000	..= 0xBFFF	=> {
				if let Some(ref ram) = self.ram {
					if self.ram_enabled {
						return ram[addr as usize - 0xA000 + (self.ram_bank as usize * 0x2000)];
					}
				}

				0xFF
			},
			_ => panic!("invalid cart read")
		}

	}

	pub fn write(&mut self, addr: u16, write: u8) {

		match addr {
			0		..= 0x3FFF	=> self.rom[addr as usize] = write,
			0x4000	..= 0x7FFF	=> self.rom[(addr as usize - 0x4000) + (0x4000 * self.rom_bank as usize)] = write,
			0xA000	..= 0xBFFF	=> {
				if let Some(ref mut ram) = self.ram {
					if self.ram_enabled {
						return ram[addr as usize - 0xA000 + (self.ram_bank as usize * 0x2000)] = write;
					}
				}
			},
			_ => panic!("invalid cart read")
		}

	}

}

pub fn create_cart(rom: Vec<u8>) -> Box<dyn MBC> {

	match rom[0x147] {
		0x0 => Box::new(mbc0::MBC0::new(rom)),
		_ => panic!("unimplenented cart type 0x{:X}", rom[0x147])
	}

}