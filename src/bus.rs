// possible off-by-one error
const ROM_BANK1_START: 		u16	= 0x0;
const ROM_BANK1_END: 		u16	= 0x3FFF;

const ROM_BANK2_START: 		u16 = 0x4000;
const ROM_BANK2_END: 		u16 = 0x7FFF;

const VRAM_START: 			u16	= 0x8000;
const VRAM_END:				u16	= 0x9FFF;

const EXTERNAL_RAM_START:	u16	= 0xA000;
const EXTERNAL_RAM_END:		u16	= 0xBFFF;

const WRAM_START:			u16	= 0xC000;
const WRAM_END:				u16	= 0xDFFF;

const ECHO_RAM_START:		u16	= 0xE000;
const ECHO_RAM_END:			u16	= 0xFDFF;

const OAM_START:			u16	= 0xFE00;
const OAM_END:				u16	= 0xFE9F;

const VOID_START:			u16	= 0xFEA0;
const VOID_END:				u16	= 0xFEFF;

const IO_START:				u16	= 0xFF00;
const IO_END:				u16	= 0xFF7F;

const HRAM_START:			u16	= 0xFF80;
const HRAM_END:				u16	= 0xFFFE;

// OBOE right here i think. i might have to add - 1 to all the end values
const IE_START:				u16	= 0xFFFE;
const IE_END:				u16	= 0xFFFF;

#[allow(dead_code)]
pub struct Bus {
	// devices on the bus
	// TODO rom bank switching
	// these are probably just going to be placeholders until the i write actual devices populate the bus

	memory: [u8; 0xFFFF]

	/*
	rom_bank1: 		[u8; ROM_BANK1_END],							// fixed ROM bank from the cart
	rom_bank2: 		[u8; ROM_BANK2_END - ROM_BANK2_START],			// swappable ROM bank from the cart
	vram: 			[u8; VRAM_END - VRAM_START],					// video RAM
	external_ram: 	[u8; EXTERNAL_RAM_END - EXTERNAL_RAM_START],	// extra external RAM exposed by the cart
	wram: 			[u8; WRAM_END - WRAM_START],					// work RAM
	echo_ram: 		[u8; ECHO_RAM_END - ECHO_RAM_START],			// a mirror of C000-DDFF; use of this area is prohibited by nintendo
	oam:			[u8; OAM_END - OAM_START],						// object attribute memory
	void:			[u8; VOID_END - VOID_START],					// not usable; use of this area is also prohibited by nintendo
	io:				[u8; IO_END - IO_START],						// I/O registers
	hram:			[u8; HRAM_END - HRAM_START],					// high RAM; faster than normal ram and used for DMA transfers
	ie:				[u8; IE_END - IE_START]							// interrupt enable register
	*/
}

impl Bus {
	pub fn new() -> Self {

		Bus {
			memory: [0; 0xFFFF]
		}

	}

	pub fn read_byte(&self, addr: u16) -> u8 {

		if addr >= ROM_BANK1_START && addr <= ROM_BANK2_END {
			return self.memory[addr as usize];
		}

		// gameboy doctor
		if addr == 0xFF44 {
			return 0x90;
		}

		0

	}

	pub fn write_byte(&mut self, addr: u16, write: u8) -> bool {

		if addr >= ROM_BANK1_START && addr < ROM_BANK2_END {
			self.memory[addr as usize] = write;
			return true;
		}

		false

	}

}