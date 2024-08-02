use std::fs;

use eframe::egui::*;

use emu::Gameboy;

pub struct Control {
	pub paused: bool,
	pub speed: u8,
	pub scale: usize,

	pub rom_path: String,
	pub rom_list: Vec<&'static str>,
	pub rom_index: usize,

	pub breakpoints: Vec<u16>,
	breakpoints_window_open: bool,
	breakpoint_str: String,
}

impl Control {

	pub fn new() -> Self {
		Self {
			paused: true,
			speed: 1,
			scale: 2,
			rom_path: "roms/dmg-acid2.gb".to_string(),
			rom_list:  vec!["roms/dmg-acid2.gb", "roms/dmg_bootrom.gb", "roms/hello-world.gb", "roms/tetris.gb", "roms/drmario.gb", "tests/cpu_instrs/cpu_instrs.gb"],
			rom_index: 0,
			breakpoints: Vec::new(),
			breakpoints_window_open: false,
			breakpoint_str: String::new(),
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
					4 => 6,
					_ => 1
				}
			}

			if ui.button("Step").clicked() {
				for _ in 0..self.speed {
					emu.tick();
				}
			}

			if ui.button("Run scanline").clicked() {
				emu.run_scanline();
			}

			
		});

		ui.horizontal(|ui| {

			if ComboBox::from_label("Select ROM").show_index(ui, &mut self.rom_index, self.rom_list.len(), |i| self.rom_list[i]).changed() {

				self.rom_path = self.rom_list[self.rom_index].to_string();

				self.reset_emu(emu);

			}

			if ui.text_edit_singleline(&mut self.rom_path).lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
				self.reset_emu(emu);
			}


		});

		ui.horizontal(|ui| {

			if ui.button("breakpoints").clicked() {
				self.breakpoints_window_open = !self.breakpoints_window_open;
			}

			if self.breakpoints_window_open {

				Window::new("Breakpoints").show(ctx, |ui| {

					let mut removed_breakpoint: Option<usize> = None;

					for (i, breakpoint) in self.breakpoints.iter().enumerate() {
						ui.horizontal(|ui| {

							ui.label(format!("PC: 0x{:04X}", breakpoint));
							
							if ui.button("Remove").clicked() {
								removed_breakpoint = Some(i);
							}

						});
					}

					if let Some(i) = removed_breakpoint {
						self.breakpoints.remove(i);
					}

					ui.horizontal(|ui| {

						if ui.text_edit_singleline(&mut self.breakpoint_str).lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
							
							if let Ok(breakpoint) = u16::from_str_radix(&self.breakpoint_str, 16) {
								self.breakpoints.push(breakpoint);
							} else {
								eprintln!("[ERROR] Unable to parse breakpoint {}", self.breakpoint_str);
							}

						}

					});

				});

			}

		});

	}

	fn reset_emu(&mut self, emu: &mut Gameboy) {

		*emu = Gameboy::new();

		let rom_open = fs::read(self.rom_path.clone());

		if let Ok(rom) = rom_open {

			emu.init(&rom);

		} else {
			eprintln!("[ERROR] failed to open rom. Error: {:?}", rom_open.unwrap_err());
		}

		

	}

}