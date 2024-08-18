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

struct Sprite {
	pos_y: i16,
	pos_x: i16,
	tile_id: u8,

	priority: bool,
	y_flip: bool,
	x_flip: bool,
	palette: bool,
}

impl Sprite {

	pub fn from_oam(base_addr: u16, oam: &mut [u8; 160]) -> Self {

		let attributes = oam[base_addr as usize + 3];

		Self {
			pos_y: oam[base_addr as usize] as i16 - 16,
			pos_x: oam[base_addr as usize + 1] as i16 - 8,
			tile_id: oam[base_addr as usize + 2],

			priority: 	attributes & 0x80 != 0,
			y_flip: 	attributes & 0x40 != 0,
			x_flip:		attributes & 0x20 != 0,
			palette:	attributes & 0x10 != 0,
		}
	}

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
		self.obj_enable				= write & 0x02 != 0;
		self.obj_size 				= write & 0x04 != 0;
		self.bg_tilemap_area 		= write & 0x08 != 0;
		self.tile_data_area 		= write & 0x10 != 0;
		self.window_enable 			= write & 0x20 != 0;
		self.window_tilemap_area 	= write & 0x40 != 0;
		self.lcd_enable 			= write & 0x80 != 0;
	}

}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GBColour {
	White = 0,
	LightGrey = 1,
	DarkGrey = 2,
	Black = 3
}

impl From<u8> for GBColour {
	
	fn from(from: u8) -> GBColour {
		match from {
			0 => GBColour::White,
			1 => GBColour::LightGrey,
			2 => GBColour::DarkGrey,
			3 => GBColour::Black,
			_ => panic!("invalid GBColour")
		}
	}

}

struct Palette {
	id_0: GBColour,
	id_1: GBColour,
	id_2: GBColour,
	id_3: GBColour,
}

impl Palette {

	pub fn new(initial_write: u8) -> Self {
		let mut pal = Self {
			id_0: GBColour::White,
			id_1: GBColour::LightGrey,
			id_2: GBColour::DarkGrey,
			id_3: GBColour::Black,
		};

		pal.write(initial_write);

		return pal;
	}

	pub fn get_pal_value(&self, id: u8) -> GBColour {

		match id {
			0 => self.id_0,
			1 => self.id_1,
			2 => self.id_2,
			3 => self.id_3,
			_ => panic!("invalid palette index")
		}

	}

	pub fn read(&self) -> u8 {
		self.id_0 as u8
			| (self.id_1 as u8) << 2
			| (self.id_2 as u8) << 4
			| (self.id_3 as u8) << 6
	}

	pub fn write(&mut self, write: u8) {
		self.id_0 = GBColour::from(write & 0b11);
		self.id_1 = GBColour::from((write >> 2) & 0b11);
		self.id_2 = GBColour::from((write >> 4) & 0b11);
		self.id_3 = GBColour::from((write >> 6) & 0b11);
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
	
	reg_bgp: Palette,	// 0xFF47: Background palette
	reg_obp0: Palette,	// 0xFF48: Object palette 0
	reg_obp1: Palette,	// 0xFF49: Object palette 1

	win_ly: u8,			// window internal line counter
	
	pub line_dots: i32, // amount of dots that has passed; reset each line.

	pub vram: [u8; 0x2000], // 8k
	oam: [u8; 160],

	pub pixel_buf: [GBColour; 144 * 160],
	pub tile_data_buf: Vec<[GBColour; 8 * 8]>
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
			reg_stat: 0x85,
			reg_lcdc: LCDC::new(0x91),

			reg_bgp: Palette::new(0xFC),
			reg_obp0: Palette::new(0),
			reg_obp1: Palette::new(0),

			win_ly: 0,

			line_dots: 0,

			vram: [0; 0x2000],
			oam: [0; 160],

			pixel_buf: [GBColour::LightGrey; 144 * 160],

			tile_data_buf: vec![[GBColour::Black; 8 * 8]; 384],
		}
	}

	// called directly after the cpu has been ticked, and updates accordingly
	pub fn tick(&mut self, cycles: u64) {

		if !self.reg_lcdc.lcd_enable {
			self.rendering_mode = RenderingMode::HBlank;
			self.reg_ly = 0;
			self.line_dots = 0;

			self.reg_stat = (self.reg_stat & !0x3) | self.rendering_mode as u8;

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

			// update window internal line counter whenever the window is active on a line
			if self.reg_lcdc.window_enable && self.reg_ly > self.reg_wy && self.reg_wx < 159 + 7 && self.reg_wy < 144 {
				self.win_ly = self.win_ly.wrapping_add(1);
			}

			// draw scanline
			if self.reg_ly < 144 {
				self.draw_scanline();
			}

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

			} else {
				self.reg_stat &= !(StatFlag::LYCcmp as u8);
			}

			// VBlank
			if self.reg_ly == 144 {

				self.rendering_mode = RenderingMode::VBlank;
				self.intf.borrow_mut().raise(InterruptFlag::VBlank);

				//println!("vblank");

				if self.reg_stat & StatFlag::Mode1Int as u8 != 0 {
					self.intf.borrow_mut().raise(InterruptFlag::LCDC)
				}
			}

			// end of VBlank
			if self.reg_ly == 154 {
				self.reg_ly = 0;
				self.win_ly = 0;
			}

		}

		// update stat register for potentially new rendering mode
		self.reg_stat = (self.reg_stat & !0x3) | self.rendering_mode as u8;

	}

	fn draw_scanline(&mut self) {

		if self.reg_lcdc.bg_enable {
			self.draw_tiles()
		} else {
			for x in 0..160 {
				self.pixel_buf[x + 160 * self.reg_ly as usize] = GBColour::White;
			}
		}

		if self.reg_lcdc.obj_enable {
			self.draw_sprites()
		}

	}

	fn draw_tiles(&mut self) {

		let (tile_data_area, sign): (u16, bool) = match self.reg_lcdc.tile_data_area {
			false => (0x9000, true),	// signed addressing uses 0x9000 as a base pointer
			true => (0x8000, false),
		};

		for x in 0..160 {

			let in_window = self.reg_lcdc.window_enable && self.reg_ly >= self.reg_wy && x >= self.reg_wx.wrapping_sub(7);

			let y_pos = match in_window {
				true => self.win_ly,
				false => self.reg_ly.wrapping_add(self.reg_scy)
			};

			let x_pos = match in_window {
				true => x.wrapping_sub(self.reg_wx.wrapping_sub(7)),
				false => x.wrapping_add(self.reg_scx)
			};

			let tilemap_area: u16 = match in_window {
				false => match self.reg_lcdc.bg_tilemap_area {
					false => 0x9800,
					true => 0x9C00,
				},
				true => match self.reg_lcdc.window_tilemap_area {
					false => 0x9800,
					true => 0x9C00,
				}
			};

			// get tile id
			let tile_id = match in_window {
				true => self.read(tilemap_area + ((y_pos as u16 / 8) * 32) + (x_pos as u16 / 8)),
				false => self.read(tilemap_area + ((y_pos as u16 / 8) * 32) + (x_pos as u16 / 8)),
			};

			// get tile data base address
			let tile_base_addr = match sign {
				false => tile_data_area + tile_id as u16 * 16,
				true => tile_data_area.wrapping_add(((tile_id as i8) as u16).wrapping_mul(16))
			};

			let tile_addr_offset = (y_pos % 8) as u16 * 2;

			let data_1 = self.read(tile_base_addr + tile_addr_offset);
			let data_2 = self.read(tile_base_addr + tile_addr_offset + 1);

			// the index of the tiles used to form the palette id for this pixel
			let pixel_index = match in_window {
				true => self.reg_wx.wrapping_sub(x) % 8,
				false => 7 - (x_pos % 8),		// the 7 is there to swap which bit is selected
			};

			let pal_id = (data_1 >> pixel_index & 1) | (data_2 >> pixel_index & 1) << 1;

			self.pixel_buf[x as usize + 160 * self.reg_ly as usize] = self.reg_bgp.get_pal_value(pal_id);

		}

	}

	pub fn draw_sprites(&mut self) {

		for oam_index in 0..40 {

			let sprite = Sprite::from_oam(oam_index * 4, &mut self.oam);
			
			let size_y = match self.reg_lcdc.obj_size {
				true => 16,
				false => 8,
			};

			// is the sprite on this line?
			if self.reg_ly as i16 >= sprite.pos_y && (self.reg_ly as i16) < (sprite.pos_y + size_y) {

				let sprite_line = (self.reg_ly as i16 - sprite.pos_y) as u16;

				let tile_data_addr = (0x8000 + sprite.tile_id as u16 * 16) + sprite_line * 2;

				let data_1 = self.read(tile_data_addr);
				let data_2 = self.read(tile_data_addr + 1);

				for x in 0..8 {

					let x_offset = sprite.pos_x + x;

					if x_offset < 0 || x_offset > 160 {
						continue;
					}

					let pal = match sprite.palette {
						false => &self.reg_obp0,
						true => &self.reg_obp1,
					};

					let pal_id = (data_1 >> x & 1) | (data_2 >> x & 1) << 1;

					let pal_value = pal.get_pal_value(pal_id);

					if pal_value != GBColour::White {
						self.pixel_buf[x_offset as usize + 160 * self.reg_ly as usize] = self.reg_bgp.get_pal_value(pal_id);
					}

				}

			}

		}

	}

	pub fn draw_tile_data(&mut self) {

		for tile in 0..384_u16 {
			
			for tile_offset in 0..8_u16 {

				let data_1 = self.read(0x8000 + tile * 16 + (tile_offset * 2));
				let data_2 = self.read(0x8000 + tile * 16 + (tile_offset * 2 + 1));

				for pixel in 0..8 {

					let pal_id = (data_1 >> pixel & 1) | (data_2 >> pixel & 1) << 1;

					self.tile_data_buf[tile as usize][(pixel + 8 * tile_offset) as usize] = self.reg_bgp.get_pal_value(pal_id);

				}

			}

		}

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
			0xFF47 => self.reg_bgp.read(),
			0xFF48 => self.reg_obp0.read(),
			0xFF49 => self.reg_obp1.read(),
			0xFF4A => self.reg_wy,
			0xFF4B => self.reg_wx,
			_ => panic!("invalid ppu read address 0x{:X}", addr)
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = write,
			0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = write,

			0xFF40 => self.reg_lcdc.write(write),
			0xFF41 => self.reg_stat = write & !(self.reg_stat & 0b111), // PPUmode and LY=LYC are read only
			0xFF42 => self.reg_scy = write,
			0xFF43 => self.reg_scx = write,
			0xFF44 => {},
			0xFF45 => self.reg_lyc = write,
			0xFF47 => self.reg_bgp.write(write),
			0xFF48 => self.reg_obp0.write(write),
			0xFF49 => self.reg_obp1.write(write),
			0xFF4A => self.reg_wy = write,
			0xFF4B => self.reg_wx = write,
			_ => panic!("invalid ppu write address 0x{:X}", addr)
		}
	}

}
