use std::{fs, time::{Duration, Instant}};

use eframe::{egui::{self, Frame, Stroke, Style, TextBuffer}, App};

use emu::Gameboy;

use crate::components::{control::Control, cpu::Cpu, ppu::Ppu, display::Display};

const CYCLES_PER_FRAME: usize = (4194304.0 / 60.0) as usize;

pub struct Debugger {
	emu: Gameboy,
	last_update: Instant,

	display: Display,

	control: Control,
	cpu: Cpu,
	ppu: Ppu
}

impl Debugger {
	pub fn new(cc: &eframe::CreationContext) -> Self {

		let mut emu = Gameboy::new();

		let cart = fs::read("roms/dmg-acid2.gb").unwrap();

		emu.init(&cart);

		Self {
			emu: emu,
			last_update: Instant::now(),

			display: Display::new(),

			control: Control::new(),
			cpu: Cpu::new(),
			ppu: Ppu::new(),
		}
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		
		if self.last_update.elapsed() >= Duration::from_secs_f64(1.0 / 60.0) && !self.control.paused {
			let mut frames = self.last_update.elapsed().as_secs_f64();
			
			while frames >= 1.0 / 60.0 {
				for _ in 0..CYCLES_PER_FRAME * self.control.speed as usize {
					self.emu.tick();
				}
				
				frames -= CYCLES_PER_FRAME as f64;
			}
			
			self.last_update = Instant::now();
		}
		
		egui::SidePanel::left("left_pannel").show(ctx, |ui| {
			
			ui.heading("gb-emu");
			
			ui.separator();
			
			self.control.show(ctx, ui, &mut self.emu);
			
			ui.separator();
			
			self.cpu.show(ctx, ui, &mut self.emu);
			
			ui.separator();
			
			self.ppu.show(ctx, ui, &mut self.emu)

		});
		
		egui::SidePanel::right("right_pannel").show(ctx, |ui| {
			
			ui.strong("VRAM Viewer");
			
			ui.separator();
			
			self.ppu.vram_viewer(ctx, ui, &mut self.emu);
			
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			self.display.show(ctx, ui, &mut self.emu, self.control.scale)
		});
		
		ctx.request_repaint();
		
	}
}