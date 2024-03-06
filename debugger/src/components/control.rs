use std::fmt::format;

use eframe::egui::*;

use emu::Gameboy;

pub struct Control {
	paused: bool,
	speed: u8,
}

impl Control {

	pub fn new() -> Self {
		Self {
			paused: false,
			speed: 1,
		}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {
		
		ui.label("\nControl");
		

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

			if ui.button("Step").clicked() {
				emu.tick();
			};
		});
	}

}