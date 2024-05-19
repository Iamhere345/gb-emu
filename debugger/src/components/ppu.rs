use std::fmt::format;

use eframe::egui::*;

use emu::{ppu, Gameboy};

pub struct Ppu;

impl Ppu {

	pub fn new() -> Self {
		Ppu {}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		ui.label("PPU");

		ui.label(format!("PPU Mode: {:?}", emu.bus.borrow().ppu.rendering_mode));

		ui.label(format!("Line dots: {}", emu.bus.borrow().ppu.line_dots));

		ui.horizontal(|ui| {
			ui.label(format!("LY: {}", emu.bus.borrow_mut().read_byte(0xFF44)));
			ui.label(format!("LYC: {}", emu.bus.borrow_mut().read_byte(0xFF45)));
		});

		ui.label(format!("STAT: {:b}", emu.bus.borrow_mut().read_byte(0xFF41)));
		ui.label(format!("LCDC: {:b}", emu.bus.borrow_mut().read_byte(0xFF40)));

	}

}