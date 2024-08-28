use eframe::{egui::*, CreationContext};

use emu::{Gameboy, ppu};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Display {
	screen_tex: TextureHandle
}

impl Display {

	pub fn new(cc: &CreationContext) -> Self {

		let screen_tex = cc.egui_ctx.load_texture(
			"Display",
			ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
			TextureOptions::NEAREST
		);

		Display {
			screen_tex: screen_tex
		}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy, scale: usize) {

		let mut display_buf = vec![Color32::default(); SCREEN_WIDTH * SCREEN_HEIGHT];

		for (i, pixel) in emu.bus.borrow().ppu.pixel_buf.iter().enumerate() {

			/*
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
			*/

			match pixel {
				ppu::GBColour::White => display_buf[i] = Color32::from_rgb(255, 255, 255),
				ppu::GBColour::LightGrey => display_buf[i] = Color32::from_rgb(128, 128, 128),
				ppu::GBColour::DarkGrey => display_buf[i] = Color32::from_rgb(64, 64, 64),
				ppu::GBColour::Black => display_buf[i] = Color32::from_rgb(0, 0, 0)
			}

		}

		let colour_image = ColorImage {
			size: [SCREEN_WIDTH, SCREEN_HEIGHT],
			pixels: display_buf,
		};

		self.screen_tex.set(colour_image, TextureOptions::NEAREST);

		let image = Image::new(&self.screen_tex);
        let image = image.fit_to_exact_size(vec2((SCREEN_WIDTH * scale) as f32, (SCREEN_WIDTH * scale) as f32));
        //image.paint_at(ui, ui.ctx().available_rect());
		ui.add(image);

	}

}