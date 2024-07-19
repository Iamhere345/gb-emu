use eframe::egui::*;

use emu::Gameboy;

pub struct Control {
	pub paused: bool,
	pub speed: u8,
	pub scale: usize,
}

impl Control {

	pub fn new() -> Self {
		Self {
			paused: true,
			speed: 1,
			scale: 2,
		}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {
		
		ui.strong("Control");
		
		ui.horizontal(|ui| {
			if ui.button(if self.paused == true { "Start" } else { "Stop" }).clicked() {
				self.paused = !self.paused;
			}

			if ui.button(format!("Speed: {}x", self.speed)).clicked() {
				self.speed = match self.speed {
					1 => 2,
					2 => 4,
					4 => 8,
					_ => 1
				}
			}

			if ui.button(format!("Scale: {}x", self.scale)).clicked() {
				self.scale = match self.scale {
					1 => 2,
					2 => 4,
					4 => 8,
					_ => 1
				}
			}

			if ui.button("Step").clicked() {
				for _ in 0..self.speed {
					emu.tick();
				}
			}

		});
	}

}