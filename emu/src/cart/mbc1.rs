use super::MBC;

pub struct MBC1 {
	rom: Vec<u8>,
	ram: Option<Vec<u8>>,

	rom_bank: u16,
	upper_bank: u8,
	banking_mode: bool,

	rom_banks: usize,
	ram_banks: usize,
	
	ram_enabled: bool,
	has_battery: bool,
}

impl MBC1 {
	
	pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
		
		let has_ram = match rom[0x147] {
			0x2 | 0x3 => true,
			_ => false,
		};

		let has_battery = rom[0x147] == 0x3;

		let ram = if has_ram { Some(vec![0; ram_size]) } else { None };
		let rom_size = rom[0x148];

		Self {
			rom: rom,
			ram: ram,
			
			rom_bank: 1,
			upper_bank: 0,
			banking_mode: false,

			rom_banks: 2 * (2 as usize).pow(rom_size as u32),
			ram_banks: ram_size >> 13,

			ram_enabled: false,
			has_battery: has_battery,
		}

	}

}

impl MBC for MBC1 {
	fn read(&self, addr: u16) -> u8 {

		match addr {
			// rom bank 0
			0		..= 0x3FFF	=> {
				let bank = if self.banking_mode {
					(self.upper_bank as usize) << 5
				} else {
					0
				} % self.rom_banks;

				self.rom[addr as usize + (bank * 0x4000)]
			},
			// rom bank x
			0x4000	..= 0x7FFF	=> {
				let bank = (self.rom_bank as usize | ((self.upper_bank as usize) << 5)) % self.rom_banks;

				self.rom[(addr as usize - 0x4000) + (0x4000 * bank)]
			},
			// ram bank x
			0xA000	..= 0xBFFF	=> {
				if let Some(ref ram) = self.ram {
					if self.ram_enabled && self.ram_banks != 0 {
						let bank = if self.banking_mode {
							self.upper_bank as usize
						} else {
							0
						} % self.ram_banks;

						return ram[addr as usize - 0xA000 + (bank * 0x2000)];
					}
				}

				0xFF
			},
			_ => panic!("invalid cart read")
		}

	}

	fn write(&mut self, addr: u16, write: u8) {

		match addr {
			// ram enable register
			0		..= 0x1FFF	=> self.ram_enabled = write & 0xF == 0xA,
			// rom bank register
			0x2000	..= 0x3FFF	=> {

				self.rom_bank = (write & 0b0001_1111) as u16;

				if self.rom_bank == 0 { self.rom_bank = 1; }

			}
			// ram/upper rom bank register
			0x4000	..= 0x5FFF	=> {
				self.upper_bank = write & 0b11;
			}
			// banking mode register
			0x6000	..=	0x7FFF	=> {
				self.banking_mode = (write & 0x1) != 0;
			}
			// ram write
			0xA000	..= 0xBFFF	=> {

				if let Some(ref mut ram) = self.ram {
					if self.ram_enabled && self.ram_banks != 0 {
						let bank = if self.banking_mode {
							self.upper_bank as usize
						} else {
							0
						} % self.ram_banks;

						return ram[addr as usize - 0xA000 + (bank * 0x2000)] = write;
					}
				}
			},
			_ => panic!("invalid cart write")
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