use std::cell::RefCell;
use std::rc::Rc;

use crate::cart::create_cart;

use super::timer::Timer;
use super::interrupt::Interrupt;
use super::ppu::PPU;
use super::joypad::Joypad;
use super::cart::MBC;

// possible off-by-one error
const ROM_BANK1_START: 		u16	= 0x0;
const ROM_BANK2_END: 		u16 = 0x7FFF;

const EXT_RAM_START:		u16 = 0xA000;
const EXT_RAM_END:			u16 = 0xBFFF;

const WRAM_START:			u16	= 0xC000;
const WRAM_END:				u16	= 0xDFFF;

const HRAM_START:			u16	= 0xFF80;
const HRAM_END:				u16	= 0xFFFE;

pub enum MemRegister {
	IE = 0xFFFF,		// interrupt enable
	IF = 0xFF0F,		// interrupt flag

	LY = 0xFF44,		// Line counter register
	LYC = 0xFF45,
	SCY = 0xFF42,
	SCX = 0xFF43,
	WX = 0xFF4B,
	WY = 0xFF4A,
	STAT = 0xFF41,
	LCDC = 0xFF40,
	BGP = 0xFF47,

	// TODO add other registers as they as needed
}

#[allow(dead_code)]
pub struct Bus {
	memory: [u8; 64 * 1024],

	pub cart: Box<dyn MBC>,

	pub intf: Rc<RefCell<Interrupt>>,
	pub timer: Timer,
	pub ppu: PPU,
	pub joypad: Joypad,
	
	dma_src: u8,

	pub rom: [u8; 0x8000],
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

	pub fn new(rom: Vec<u8>) -> Self {

		let intf = Rc::new(RefCell::new(Interrupt::default()));

		Bus {

			cart: create_cart(rom),

			memory: [0xFF; 64 * 1024],

			intf: Rc::clone(&intf),
			timer: Timer::new(Rc::clone(&intf)),
			ppu: PPU::new(Rc::clone(&intf)),
			joypad: Joypad::new(Rc::clone(&intf)),

			dma_src: 0,

			rom: [0xFF; 0x8000],
			wram: [0xFF; 0x8000],
			hram: [0xFF; 0x7F],
		}

	}

	pub fn read_byte(&self, addr: u16) -> u8 {

		//let logo = [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E];

		return match addr {

			//0x104			..=	0x133 => logo[addr as usize - 0x104],

			ROM_BANK1_START	..=	ROM_BANK2_END => self.cart.read(addr),

			EXT_RAM_START	..= EXT_RAM_END => self.cart.read(addr),

			WRAM_START		..=	WRAM_END => self.wram[(addr - WRAM_START) as usize],

			/* PPU addresses */
			0x8000			..= 0x9FFF => self.ppu.read(addr),
			0xFE00			..=	0xFE9F => self.ppu.read(addr),

			0xFF00			=> self.joypad.read(),

			0xFF46			=> self.dma_src,

			0xFF40 			..= 0xFF4B => self.ppu.read(addr),
			
			0xFF04			..= 0xFF07 => self.timer.read(addr),
			0xFF0F			|	0xFFFF => self.intf.borrow().read(addr),

			HRAM_START		..= HRAM_END => self.hram[(addr - HRAM_START) as usize],
			_ => self.memory[addr as usize]
		};

	}

	pub fn write_byte(&mut self, addr: u16, write: u8) {

		//self.memory[addr as usize] = write;
		//return;

		match addr {

			ROM_BANK1_START	..=	ROM_BANK2_END => self.cart.write(addr, write),

			EXT_RAM_START	..= EXT_RAM_END => self.cart.write(addr, write),

			WRAM_START		..=	WRAM_END => self.wram[(addr - WRAM_START) as usize] = write,

			/* PPU addresses */
			0x8000			..= 0x9FFF => self.ppu.write(addr, write),
			0xFE00			..=	0xFE9F => self.ppu.write(addr, write),

			0xFF00			=> self.joypad.write(write),

			0xFF46 => self.dma_transfer(write),

			0xFF40			..= 0xFF4B => self.ppu.write(addr, write),

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

	pub fn dma_transfer(&mut self, src: u8) {

		self.dma_src = src;

		let addr: u16 = (src as u16) << 8;	// addr is src * 100

		for i in 0x0..0xA0 {
			self.write_byte(0xFE00 + i, self.read_byte(addr + i))
		}

	}

	pub fn clear_test_mem(&mut self) {
		for byte in self.memory.iter_mut() { *byte = 0 }
	}

}