use eframe::{egui, App};

use emu::Gameboy;

use crate::components::control::Control;

pub struct Debugger {
	emu: Gameboy,

	control: Control
}

impl Debugger {
	pub fn new(cc: &eframe::CreationContext) -> Self {
		Self {
			emu: Gameboy::new(),

			control: Control::new()
		}
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

		egui::SidePanel::left("left_pannel").show(ctx, |ui| {
			self.control.show(ctx, ui, &mut self.emu);

			ui.separator();
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label("Hello, World!");
		});
	}
}