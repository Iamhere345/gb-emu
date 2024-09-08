use std::fs;
use eframe::egui::*;
use native_dialog::{FileDialog, Filter};

use emu::Gameboy;

pub struct Control {
	pub paused: bool,
	pub speed: u8,
	pub scale: usize,

	pub rom_path: String,
	old_rom_path: String,

	pub rom_list: Vec<&'static str>,
	pub rom_index: usize,

	enable_bootrom: bool,

	pub breakpoints: Vec<u16>,
	breakpoints_window_open: bool,
	breakpoint_str: String,
}

impl Control {

	pub fn new() -> Self {
		Self {
			paused: true,
			speed: 1,
			scale: 4,

			rom_path: "roms/dmg-acid2.gb".to_string(),
			old_rom_path: "roms/dmg-acid2.gb".to_string(),
			rom_list:  vec![
				"roms/dmg-acid2.gb", 
				
				"roms/games/tetris.gb", 
				"roms/games/drmario.gb", 
				"roms/games/sml.gb", 
				"roms/games/pokered.gb",
				"roms/games/tennis.gb",
				"roms/games/megaman.gb",
				"roms/games/zelda.gb",
				"roms/games/metroid.gb",
				"roms/games/dk3.gb",
				"roms/games/harvest_moon.gb",

				"roms/hello-world.gb",

				"tests/blargg/cpu_instrs.gb",
				],
			rom_index: 0,

			enable_bootrom: false,

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
					emu.cycles += emu.tick();
				}
			}

			if ui.button("Run scanline").clicked() {
				emu.run_scanline();
			}

			
		});

		ui.horizontal(|ui| {

			if ComboBox::from_label("").show_index(ui, &mut self.rom_index, self.rom_list.len(), |i| self.rom_list[i]).changed() {

				self.rom_path = self.rom_list[self.rom_index].to_string();

				self.save_sram(emu);
				self.reset_emu(emu, self.enable_bootrom);

				self.old_rom_path = self.rom_path.clone();

			}

			if ui.button("Select ROM").clicked() {

				let rom_path = FileDialog::new()
					.set_location(std::env::current_dir().unwrap().as_path())
					.add_filter("GB Rom", &["gb", "gbc"])
					.show_open_single_file()
					.unwrap();

				if let Some(rom) = rom_path {
					self.rom_path = rom.to_str().unwrap().to_string();

					self.save_sram(emu);
					self.reset_emu(emu, self.enable_bootrom);

					self.old_rom_path = self.rom_path.clone();
				}

			}

			ui.checkbox(&mut self.enable_bootrom, "Enable Bootrom");

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

	pub fn save_sram(&self, emu: &Gameboy) {
		if emu.bus.borrow().cart.is_battery_backed() {
			let save_path = format!("{}.sav", self.old_rom_path);
			let sram = emu.bus.borrow().cart.dump_sram();

			fs::write(save_path, &sram).expect("Oh no! Your progress couldn't be saved.");

		}
	}

	pub fn load_sram(&self, emu: &mut Gameboy) {
		if emu.bus.borrow().cart.is_battery_backed() {

			let save_path = format!("{}.sav", self.rom_path);
			let sram_res = fs::read(save_path);

			if let Ok(sram) = sram_res {
				emu.bus.borrow_mut().cart.load_sram(sram);
			} else {
				println!("no save file present for {}", self.rom_path);
			}

		}
	}

	fn reset_emu(&mut self, emu: &mut Gameboy, enable_bootrom: bool) {

		let rom_open = fs::read(self.rom_path.clone());

		if let Ok(rom) = rom_open {

			*emu = Gameboy::new(rom);

			if enable_bootrom {
				emu.bus.borrow_mut().bootrom_loaded = true;
				emu.cpu.pc = 0;
			}

			self.load_sram(emu);

		} else {
			eprintln!("[ERROR] failed to open rom. Error: {:?}", rom_open.unwrap_err());
		}

	}

}