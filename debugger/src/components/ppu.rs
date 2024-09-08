use eframe::egui::*;

use emu::{ppu, Gameboy};

pub struct Ppu;

impl Ppu {

	pub fn new() -> Self {
		Ppu {}
	}

	pub fn show(&mut self, _ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		ui.strong("PPU");

		ui.monospace(format!("PPU Mode: {:?}", emu.bus.borrow().ppu.rendering_mode));

		ui.monospace(format!("Line dots: {}", emu.bus.borrow().ppu.line_dots));

		ui.horizontal(|ui| {
			ui.monospace(format!("LY: {}", emu.bus.borrow().read_byte(0xFF44)));
			ui.monospace(format!("LYC: {}", emu.bus.borrow().read_byte(0xFF45)));
		});

		ui.monospace(format!("STAT: {:#010b}", emu.bus.borrow().read_byte(0xFF41)));
		ui.monospace(format!("LCDC: {:#010b}", emu.bus.borrow().read_byte(0xFF40)));
		ui.monospace(format!("BGP:  {:#010b}", emu.bus.borrow().read_byte(0xFF47)));

	}

	pub fn vram_viewer(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		emu.bus.borrow_mut().ppu.draw_tile_data();

		let tile_data_buf = emu.bus.borrow().ppu.tile_data_buf.clone();

		let mut vram_viewer_buf: Vec<Color32> = vec![Color32::GREEN; 16 * 8 * 24 * 8];
		
		for row in 0..16 {
			for col in 0..24 {
				let tile = tile_data_buf[col + 24 * row];

				for y in 0..8 {
					for x in 0..8 {
						let i = y * 8 + x;
						let global_x = x + 8 * col;
						let global_y = y + 8 * row;

						match tile[i] {
							ppu::GBColour::White => vram_viewer_buf[global_x + 192 * global_y] = Color32::from_rgb(255, 255, 255),
							ppu::GBColour::LightGrey => vram_viewer_buf[global_x + 192 * global_y] = Color32::from_rgb(128, 128, 128),
							ppu::GBColour::DarkGrey => vram_viewer_buf[global_x + 192 * global_y] = Color32::from_rgb(64, 64, 64),
							ppu::GBColour::Black => vram_viewer_buf[global_x + 192 * global_y] = Color32::from_rgb(0, 0, 0),
						}
					}
				}
			}
		}

		// draw grid
		for row in 0..16 {
			for col in 0..24 * 8 {
				vram_viewer_buf[col + 192 * row * 8] = Color32::from_rgba_premultiplied(255, 0, 0, 128);
			}
		}

		for col in 0..24 {
			for row in 0..16 * 8 {
				vram_viewer_buf[col * 8 + 192 * row] = Color32::from_rgba_premultiplied(255, 0, 0, 128);
			}
		}

		let colour_image = ColorImage {
			size: [24 * 8, 16 * 8],
			pixels: vram_viewer_buf,
		};

		let tex = ctx.load_texture("VRAM Viewer", colour_image, TextureOptions::LINEAR);

		ui.image((tex.id(), tex.size_vec2()));

	}

}