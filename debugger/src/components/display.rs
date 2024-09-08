use eframe::{egui::*, CreationContext};

use emu::{Gameboy, ppu};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Display {
	screen_tex: TextureHandle,
}

impl Display {

	pub fn new(cc: &CreationContext) -> Self {

		let screen_tex = cc.egui_ctx.load_texture(
			"Display",
			ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
			TextureOptions::NEAREST
		);

		Display {
			screen_tex: screen_tex,
		}
	}

	pub fn show(&mut self, _ctx: &Context, ui: &mut Ui, emu: &mut Gameboy, scale: usize) {

		let mut display_buf = vec![Color32::default(); SCREEN_WIDTH * SCREEN_HEIGHT];

		for (i, pixel) in emu.bus.borrow().ppu.get_frame().iter().enumerate() {

			match pixel {
				ppu::GBColour::White => display_buf[i] = Color32::from_rgb(0xFF, 0xFF, 0xFF),
				ppu::GBColour::LightGrey => display_buf[i] = Color32::from_rgb(0xAA, 0xAA, 0xAA),
				ppu::GBColour::DarkGrey => display_buf[i] = Color32::from_rgb(0x55, 0x55, 0x55),
				ppu::GBColour::Black => display_buf[i] = Color32::from_rgb(0x0, 0x0, 0x0)
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