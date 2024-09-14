use super::MBC;

pub struct MBC2 {
	rom: Vec<u8>,
	ram: Vec<u8>,

	rom_bank: u8,

	rom_banks: usize,
	
	ram_enabled: bool,
	has_battery: bool,
}

impl MBC2 {
	
	pub fn new(rom: Vec<u8>) -> Self {

		let has_battery = match rom[0x147] {
			0x06 => true,
			_ => false,
		};

		let rom_size = rom[0x148];

		Self {
			rom: rom,
			ram: vec![0xFF; 512],
			
			rom_bank: 1,
			rom_banks: 2 * (2 as usize).pow(rom_size as u32),

			ram_enabled: false,
			has_battery: has_battery,
		}

	}

}

impl MBC for MBC2 {
	fn read(&self, addr: u16) -> u8 {

		match addr {
			// rom bank 0
			0		..= 0x3FFF	=> self.rom[addr as usize],
			// rom bank x
			0x4000	..= 0x7FFF	=> self.rom[(addr as usize - 0x4000) + (0x4000 * (self.rom_bank as usize % self.rom_banks))],
			// ram bank x
			// only the lower 4 bits of ram are accessible
			// MBC2 only reads the lower 9 bits of addr
			0xA000	..=	0xBFFF	=> if self.ram_enabled { self.ram[((addr & 0x1FF)) as usize] | 0xF0 } else { 0xFF },

			_ => panic!("invalid cart read 0x{:X}", addr)
		}

	}

	fn write(&mut self, addr: u16, write: u8) {

		match addr {
			0x0000	..=	0x3FFF	=> {
				if addr & 0x100 == 0 {
					self.ram_enabled = write & 0xF == 0x0A;
				} else {
					self.rom_bank = write & 0x0F;

					if self.rom_bank == 0 { self.rom_bank = 1 }
				}
			}
			// ram write
			0xA000	..= 0xBFFF	if self.ram_enabled => self.ram[((addr & 0x1FF)) as usize] = write | 0xF0,
			_ => {}
		}

	}

	fn is_battery_backed(&self) -> bool {
		self.has_battery
	}

	fn dump_sram(&self) -> Vec<u8> {
		self.ram.clone()
	}

	fn load_sram(&mut self, sram: Vec<u8>) {
		self.ram = sram;
	}
}