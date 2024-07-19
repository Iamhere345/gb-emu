use eframe::egui::*;

use emu::{Gameboy, ppu};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Display {}

impl Display {

	pub fn new() -> Self {
		Display {}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy, scale: usize) {

		let mut display_buf = vec![Color32::default(); SCREEN_WIDTH * scale * SCREEN_HEIGHT * scale];

		for (i, pixel) in emu.bus.borrow().ppu.pixel_buf.iter().enumerate() {

			for scaled_y in 0..scale {
				for scaled_x in 0..scale {

					let x = (i % SCREEN_WIDTH * scale) + scaled_x;
					let y = (i / SCREEN_WIDTH * scale) + scaled_y;

					match pixel {
						ppu::GBColour::White => display_buf[x + (SCREEN_WIDTH * scale) * y] = Color32::from_rgb(255, 255, 255),
						ppu::GBColour::LightGrey => display_buf[x + (SCREEN_WIDTH * scale) * y] = Color32::from_rgb(128, 128, 128),
						ppu::GBColour::DarkGrey => display_buf[x + (SCREEN_WIDTH * scale) * y] = Color32::from_rgb(64, 64, 64),
						ppu::GBColour::Black => display_buf[x + (SCREEN_WIDTH * scale) * y] = Color32::from_rgb(0, 0, 0)
					}
				}
			}

			/*match pixel {
				ppu::GBColour::White => display_buf[i] = Color32::from_rgb(255, 255, 255),
				ppu::GBColour::LightGrey => display_buf[i] = Color32::from_rgb(128, 128, 128),
				ppu::GBColour::DarkGrey => display_buf[i] = Color32::from_rgb(64, 64, 64),
				ppu::GBColour::Black => display_buf[i] = Color32::from_rgb(0, 0, 0)
			}*/		

		}

		let colour_image = ColorImage {
			size: [SCREEN_WIDTH * scale, SCREEN_HEIGHT * scale],
			pixels: display_buf,
		};

		let tex = ctx.load_texture("Display", colour_image, TextureOptions::NEAREST);

		ui.image((tex.id(), tex.size_vec2()));

	}

}