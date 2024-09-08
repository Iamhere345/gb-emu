use super::MBC;

pub struct MBC5 {
	rom: Vec<u8>,
	ram: Option<Vec<u8>>,

	rom_bank: u8,
	upper_rom_bank: u8,

	ram_bank: u8,

	rom_banks: usize,
	ram_banks: usize,
	
	ram_enabled: bool,
	has_battery: bool,
}

impl MBC5 {
	
	pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {

		let has_ram = match rom[0x147] {
			0x1A | 0x1B | 0x1D | 0x1E => true,
			_ => false,
		};

		let has_battery = match rom[0x147] {
			0x1B | 0x1E => true,
			_ => false,
		};

		let ram = if has_ram { Some(vec![0; ram_size]) } else { None };
		let rom_size = rom[0x148];

		Self {
			rom: rom,
			ram: ram,
			
			rom_bank: 1,
			upper_rom_bank: 0,

			ram_bank: 0,

			rom_banks: 2 * (2 as usize).pow(rom_size as u32),
			ram_banks: ram_size >> 13,

			ram_enabled: false,
			has_battery: has_battery,
		}

	}

}

impl MBC for MBC5 {
	fn read(&self, addr: u16) -> u8 {

		match addr {
			// rom bank 1
			0		..= 0x3FFF	=> self.rom[addr as usize],
			// rom bank x
			0x4000	..= 0x7FFF	=> {
				let bank = (((self.upper_rom_bank as usize) << 8) | self.rom_bank as usize) % self.rom_banks;

				self.rom[(addr as usize - 0x4000) + (0x4000 * bank)]
			},
			// ram bank x
			0xA000	..= 0xBFFF	=> {
				if let Some(ref ram) = self.ram {
					if self.ram_enabled && self.ram_banks != 0 {
						let bank = self.ram_bank as usize % self.ram_banks;

						return ram[addr as usize - 0xA000 + (bank * 0x2000)];
					}
				}

				println!("invalid ram read");

				0xFF
			},
			_ => panic!("invalid cart read 0x{:X}", addr)
		}

	}

	fn write(&mut self, addr: u16, write: u8) {

		match addr {
			// ram enable register
			0		..= 0x1FFF	=> self.ram_enabled = write & 0xF == 0xA,
			// rom bank register
			0x2000	..= 0x2FFF	=> self.rom_bank = write,
			// rom bank x (upper 9th bit)
			0x3000	..= 0x3FFF	=> self.upper_rom_bank = write & 0x1,
			// ram bank register
			0x4000	..= 0x5FFF	=> self.ram_bank = write & 0xF,
			// ram write
			0xA000	..= 0xBFFF	=> {

				if let Some(ref mut ram) = self.ram {
					if self.ram_enabled && self.ram_banks != 0 {
						let bank = self.ram_bank as usize % self.ram_banks;

						return ram[addr as usize - 0xA000 + (bank * 0x2000)] = write;
					}
				}
			},
			_ => {}
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