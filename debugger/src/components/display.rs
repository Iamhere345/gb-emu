use eframe::egui::*;

use emu::{Gameboy, ppu};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Display {
	scale: usize
}

impl Display {

	pub fn new() -> Self {
		Display {
			scale: 1
		}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		let mut display_buf: [Color32; SCREEN_WIDTH * SCREEN_HEIGHT] = [Color32::from_rgb(0, 0, 0); SCREEN_WIDTH * SCREEN_HEIGHT];

		for (i, pixel) in emu.bus.borrow().ppu.pixel_buf.iter().enumerate() {

			match pixel {
				ppu::GBColour::White => display_buf[i] = Color32::from_rgb(0, 0, 0),
				ppu::GBColour::LightGrey => display_buf[i] = Color32::from_rgb(64, 64, 64),
				ppu::GBColour::DarkGrey => display_buf[i] = Color32::from_rgb(128, 128, 128),
				ppu::GBColour::Black => display_buf[i] = Color32::from_rgb(255, 0, 0)
			}

		}

		let colour_image = ColorImage {
			size: [SCREEN_WIDTH, SCREEN_HEIGHT],
			pixels: display_buf.to_vec(),
		};

		let tex = ctx.load_texture("Display", colour_image, TextureOptions::LINEAR);

		ui.image((tex.id(), tex.size_vec2()));



	}

}