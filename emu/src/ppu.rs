use std::cell::RefCell;
use std::rc::Rc;

use super::interrupt::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RenderingMode {
	HBlank 	= 0,    // Mode 0
	VBlank 	= 1,    // Mode 1
	OAMscan = 2,    // Mode 2
	Draw 	= 3,	// Mode 3   
}

enum StatFlag {
	PPUMode		= 0x03,
	LYCcmp		= 0x04,
	Mode0Int	= 0x08,
	Mode1Int	= 0x10,
	Mode2Int	= 0x20,
	LYCInt		= 0x40,
}

struct LCDC {
	lcd_enable: bool,			// 0: off 1: on
	window_tilemap_area: bool,	// 0: 0x9800-0x9BFF 1: 0x9C00-0x9FFF
	window_enable: bool,		// 0: on 1: on
	tile_data_area: bool,		// (for both bg & window) 0: 0x8800-0x97FF 1: 0x8000-0x8FFF
	bg_tilemap_area: bool,		// 0: 0x9800-0x9BFF 1: 0x9C00-0x9FFF
	obj_size: bool,				// 0: 8x8 1: 8x16
	obj_enable: bool,			// 0: objects disabled 1: objects enabled
	bg_enable: bool				// 0: bg and window are disabled (even if window_enable is 1)
}

impl LCDC {

	pub fn new(initial_write: u8) -> Self {
		let mut lcdc = LCDC {
			lcd_enable: false,
			window_tilemap_area: false,
			window_enable: false,
			tile_data_area: false,
			bg_tilemap_area: false,
			obj_size: false,
			obj_enable: false,
			bg_enable: false
		};

		lcdc.write(initial_write);

		lcdc
	}

	pub fn read(&self) -> u8 {
		(if self.bg_enable		 			{ 0x01 } else { 0 })
            | (if self.obj_enable 			{ 0x02 } else { 0 })
            | (if self.obj_size 			{ 0x04 } else { 0 })
            | (if self.bg_tilemap_area 		{ 0x08 } else { 0 })
            | (if self.tile_data_area 		{ 0x10 } else { 0 })
            | (if self.window_enable 		{ 0x20 } else { 0 })
            | (if self.window_tilemap_area 	{ 0x40 } else { 0 })
            | (if self.lcd_enable 			{ 0x80 } else { 0 })
	}

	pub fn write(&mut self, write: u8) {
		self.bg_enable		 		= write & 0x01 != 0;
		self.obj_enable 			= write & 0x02 != 0;
		self.obj_size 				= write & 0x04 != 0;
		self.bg_tilemap_area 		= write & 0x08 != 0;
		self.tile_data_area 		= write & 0x010 != 0;
		self.window_enable 			= write & 0x020 != 0;
		self.window_tilemap_area 	= write & 0x040 != 0;
		self.lcd_enable 			= write & 0x080 != 0;
	}

}

pub struct PPU {
	pub rendering_mode: RenderingMode,

	intf: Rc<RefCell<Interrupt>>,

	reg_scy: u8,		// 0xFF42: Y scroll register
	reg_scx: u8,		// 0xFF43: X scroll register
	reg_wy: u8,			// 0xFF4A: window Y position
	reg_wx: u8,			// 0xFF4B: window X position - 7

	reg_ly: u8,         // 0xFF44: amount of lines drawn this frame, also the LY register
	reg_lyc: u8,        // 0xFF45: if lyc == ly and LYC=LY in STAT register set, raise interrupt
	reg_stat: u8,		// 0xFF41: LCD status register. Can be used to raise interrupts at various stages of rendering
	reg_lcdc: LCDC,		// 0xFF40: LCD control register. Can be used to alter the behaviour of the LCD and PPU

	pub line_dots: i32, // amount of dots that has passed; reset each line.

	vram: [u8; 0x2000], // 8k
	oam: [u8; 160]
}

impl PPU {

	pub fn new(intf: Rc<RefCell<Interrupt>>) -> Self {
		Self {
			rendering_mode: RenderingMode::VBlank,

			intf: intf,

			reg_scy: 0,
			reg_scx: 0,
			reg_wy: 0,
			reg_wx: 0,

			reg_ly: 0,
			reg_lyc: 0,
			reg_stat: 0,
			reg_lcdc: LCDC::new(0),

			line_dots: 0,

			vram: [0; 0x2000],
			oam: [0; 160]
		}
	}

	// called directly after the cpu has been ticked, and updates accordingly
	pub fn tick(&mut self, cycles: u64) {

		if !self.reg_lcdc.lcd_enable {
			self.rendering_mode = RenderingMode::VBlank;
			self.reg_ly = 0;
			self.line_dots = 0;

			return;
		}

		self.line_dots += cycles as i32; // 1 T-state == 1 dot

		/* Rendering mode logic */

		// OAM scan (mode 2)
		if self.line_dots <= 80 {
			if self.reg_stat & StatFlag::Mode2Int as u8 != 0 && self.rendering_mode != RenderingMode::OAMscan {
				self.intf.borrow_mut().raise(InterruptFlag::LCDC)
			}
			
			self.rendering_mode = RenderingMode::OAMscan;

		// Draw (mode 3)
		} else if self.line_dots <= 80 + 172 {
			self.rendering_mode = RenderingMode::Draw;
		
		// HBlank (Mode 0)
		} else if self.line_dots > 80 + 172 {
			self.rendering_mode = RenderingMode::HBlank;

			if self.reg_stat & StatFlag::Mode0Int as u8 != 0 && self.rendering_mode != RenderingMode::HBlank {
				self.intf.borrow_mut().raise(InterruptFlag::LCDC)
			}
		}

		// new line
		if self.line_dots >= 456 {

			let dots_carry = if self.line_dots > 0 { self.line_dots - 456 } else { 0 };

			// go to the next line and reset the line counter
			self.reg_ly += 1;
			self.line_dots = dots_carry;

			// LYC
			if self.reg_ly == self.reg_lyc {
				self.reg_stat |= StatFlag::LYCcmp as u8;

				if self.reg_stat & StatFlag::LYCInt as u8 != 0 {
					self.intf.borrow_mut().raise(InterruptFlag::LCDC);
				}
			}

			// VBlank
			if self.reg_ly == 144 {

				self.rendering_mode = RenderingMode::VBlank;
				self.intf.borrow_mut().raise(InterruptFlag::VBlank);

				if self.reg_stat & StatFlag::Mode1Int as u8 != 0 {
					self.intf.borrow_mut().raise(InterruptFlag::LCDC)
				}
			}

			// end of VBlank
			if self.reg_ly == 154 {
				self.reg_ly = 0;
			}

		}

		// update stat register for potentially new rendering mode
		self.reg_stat = self.reg_stat & self.rendering_mode as u8;

	}

	fn draw_scanline(&mut self) {

		if self.reg_lcdc.bg_enable {
			self.draw_tiles()
		}

		if self.reg_lcdc.obj_enable {
			// todo
		}

	}

	fn draw_tiles(&mut self) {

		let draw_window = if self.reg_lcdc.window_enable && self.reg_wy <= self.reg_ly { true } else { false };
		let mut index_signed = false;

		let tile_data: u16 = match self.reg_lcdc.tile_data_area {
			true => 0x8000,
			false => {
				// indexes into the 0x8800-0x0x97FF are signed
				index_signed = true;

				0x8800
			}
		};
		let tilemap_area: u16 = if draw_window {
			match self.reg_lcdc.window_tilemap_area {
				true => 0x9C00,
				false => 0x9800,
			}
		} else {
			match self.reg_lcdc.bg_tilemap_area {
				true => 0x9C00,
				false => 0x9800,
			}
		};

		let pos_y: u8 = match draw_window {
			true => self.reg_ly - self.reg_wy,
			false => self.reg_scy + self.reg_ly,
		};

		
		let tile_row: u16 = ((pos_y / 8) * 32) as u16;



	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
			0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],

			0xFF40 => self.reg_lcdc.read(),
			0xFF41 => self.reg_stat,
			0xFF42 => self.reg_scy,
			0xFF43 => self.reg_scx,
			0xFF44 => self.reg_ly,
			0xFF45 => self.reg_lyc,
			0xFF4A => self.reg_wy,
			0xFF4B => self.reg_wx,
			_ => panic!("invalid ppu read address")
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = write,
			0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = write,

			0xFF40 => self.reg_lcdc.write(write),
			0xFF41 => self.reg_stat = write & (self.reg_stat & 0b111),
			0xFF42 => self.reg_scy = write,
			0xFF43 => self.reg_scx = write,
			0xFF45 => self.reg_lyc = write,
			0xFF4A => self.reg_wy = write,
			0xFF4B => self.reg_wx = write,
			_ => panic!("invalid ppu write address")
		}
	}

}
