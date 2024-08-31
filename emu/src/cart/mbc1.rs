use super::MBC;

pub struct MBC1 {
	rom: Vec<u8>,
	ram: Option<Vec<u8>>,

	rom_bank: u16,
	ram_bank: u8,
	
	ram_enabled: bool,
	has_battery: bool,
}

impl MBC1 {
	
	pub fn new(rom: Vec<u8>, has_ram: bool, has_battery: bool, ram_size: usize) -> Self {
		
		let ram = if has_ram { Some(vec![0; ram_size]) } else { None };

		Self {
			rom: rom,
			ram: ram,
			
			rom_bank: 1,
			ram_bank: 0,

			ram_enabled: false,
			has_battery: has_battery,
		}

	}

}

impl MBC for MBC1 {
	fn read(&self, addr: u16) -> u8 {

		match addr {
			0		..= 0x3FFF	=> self.rom[addr as usize],
			0x4000	..= 0x7FFF	=> self.rom[(addr as usize % 0x4000) + (0x4000 * self.rom_bank as usize)],
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

	fn write(&mut self, addr: u16, write: u8) {

		match addr {
			0		..= 0x1FFF	=> self.ram_enabled = write == 0xA,
			0x2000	..= 0x3FFF	=> {

				self.rom_bank = (write & 0b0001_1111) as u16;

				if self.rom_bank == 0 { self.rom_bank = 1; }

			},
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

	fn is_battery_backed(&self) -> bool {
		self.has_battery
	}

	fn dump_sram(&self) -> Vec<u8> {
		if let Some(ref ram) = self.ram {
			ram.clone()
		} else {
			panic!("attempt to dump sram when no sram is present");
		}
	}

	fn load_sram(&mut self, sram: Vec<u8>) {
		self.ram = Some(sram);
	}
}