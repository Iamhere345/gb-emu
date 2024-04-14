use std::cell::{Ref, RefCell};
use std::rc::Rc;

use super::timer::Timer;
use super::interrupt::Interrupt;

// possible off-by-one error
const ROM_BANK1_START: 		u16	= 0x0;
const ROM_BANK1_END: 		u16	= 0x3FFF;

const ROM_BANK2_END: 		u16 = 0x7FFF;

const WRAM_START:			u16	= 0xC000;
const WRAM_END:				u16	= 0xDFFF;

const HRAM_START:			u16	= 0xFF80;
const HRAM_END:				u16	= 0xFFFE;

pub enum MemRegister {
	IE = 0xFFFF,		// interrupt enable
	IF = 0xFF0F,		// interrupt flag

	// TODO add other registers as they as needed
}

#[allow(dead_code)]
pub struct Bus {
	memory: [u8; 64 * 1024],

	pub intf: Rc<RefCell<Interrupt>>,
	pub timer: Timer,

	wram: [u8; 0x8000],
	hram: [u8; 0x7F],

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

		let intf = Rc::new(RefCell::new(Interrupt::default()));

		Bus {
			memory: [0; 64 * 1024],

			intf: Rc::clone(&intf),
			timer: Timer::new(Rc::clone(&intf)),

			wram: [0; 0x8000],
			hram: [0; 0x7F],
		}

	}

	pub fn read_byte(&self, addr: u16) -> u8 {

		return match addr {
			ROM_BANK1_START	..=	ROM_BANK1_END => self.memory[addr as usize],
			WRAM_START		..=	WRAM_END => self.wram[(addr - WRAM_START) as usize],

			0xFF04			..= 0xFF07 => self.timer.read(addr),
			0xFF0F			|	0xFFFF => self.intf.borrow_mut().read(addr),

			HRAM_START		..= HRAM_END => self.hram[(addr - HRAM_START) as usize],
			0xFF44 => 0x90,
			_ => self.memory[addr as usize]
		};

	}

	pub fn write_byte(&mut self, addr: u16, write: u8) {

		//self.memory[addr as usize] = write;
		//return;

		match addr {
			ROM_BANK1_START	..=	ROM_BANK2_END => self.memory[addr as usize] = write,
			WRAM_START		..=	WRAM_END => self.wram[(addr - WRAM_START) as usize] = write,

			0xFF04			..= 0xFF07 => self.timer.write(addr, write),
			0xFF0F			|	0xFFFF => self.intf.borrow_mut().write(addr, write),

			HRAM_START		..=	HRAM_END => self.hram[(addr - HRAM_START) as usize] = write,
			_ => self.memory[addr as usize] = write,
		}

	}

	pub fn read_register(&self, register: MemRegister) -> u8 {
		self.read_byte(register as u16)
	}

	pub fn write_register(&mut self, register: MemRegister, write: u8) {
		self.write_byte(register as u16, write)
	}

	pub fn clear_test_mem(&mut self) {
		for byte in self.memory.iter_mut() { *byte = 0 }
	}

}