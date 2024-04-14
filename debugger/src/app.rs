use eframe::{egui, App};

use emu::Gameboy;

use crate::components::{control::Control, cpu::Cpu};

pub struct Debugger {
	emu: Gameboy,

	control: Control,
	cpu: Cpu,
}

impl Debugger {
	pub fn new(cc: &eframe::CreationContext) -> Self {

		let mut emu = Gameboy::new();
		emu.init(include_bytes!("../../tests/instr_timing/instr_timing.gb"));

		Self {
			emu: emu,

			control: Control::new(),
			cpu: Cpu::new(),
		}
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

		egui::SidePanel::left("left_pannel").show(ctx, |ui| {

			ui.heading("gb-emu");

			ui.separator();

			self.control.show(ctx, ui, &mut self.emu);

			ui.separator();

			self.cpu.show(ctx, ui, &mut self.emu);

			ui.separator();

		});

		
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label("Hello, World!");
		});
		
		// run an instruction every update (which is every vsync, so it depends on the monitor refresh rate)
		// this temporary; eventually this will be PPU or APU synced
		if !self.control.paused {
			for _ in 0..self.control.speed {
				self.emu.tick();
			}
		}
		
		ctx.request_repaint();

	}
}