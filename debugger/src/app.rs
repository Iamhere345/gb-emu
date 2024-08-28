use std::{fs, time::{Duration, Instant}};

use eframe::{egui::{self, Frame, Key, Stroke, Style, TextBuffer}, App};

use emu::Gameboy;
use emu::joypad::*;

use crate::components::{control::{self, Control}, cpu::Cpu, display::Display, ppu::Ppu};

const CYCLES_PER_FRAME: usize = (4194304.0 / 60.0) as usize;

const BTN_A: Key 		= Key::Z;
const BTN_B: Key 		= Key::X;
const BTN_START: Key 	= Key::Enter;
const BTN_SELECT: Key 	= Key::Backspace;

const DPAD_UP: Key 		= Key::ArrowUp;
const DPAD_DOWN: Key 	= Key::ArrowDown;
const DPAD_LEFT: Key 	= Key::ArrowLeft;
const DPAD_RIGHT: Key 	= Key::ArrowRight;


pub struct Debugger {
	emu: Gameboy,
	last_update: Instant,

	display: Display,

	control: Control,
	cpu: Cpu,
	ppu: Ppu,
}

impl Debugger {
	pub fn new(cc: &eframe::CreationContext) -> Self {

		let cart = fs::read("roms/dmg-acid2.gb").unwrap();

		let mut emu = Gameboy::new(cart);

		Self {
			emu: emu,
			last_update: Instant::now(),

			display: Display::new(cc),

			control: Control::new(),
			cpu: Cpu::new(),
			ppu: Ppu::new(),
		}
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		
		ctx.input(|input| {

			let joypad = &mut self.emu.bus.borrow_mut().joypad;

			if input.key_down(BTN_A) { joypad.btn_down(GBInput::BtnA) } else { joypad.btn_up(GBInput::BtnA) }
			if input.key_down(BTN_B) { joypad.btn_down(GBInput::BtnB) } else { joypad.btn_up(GBInput::BtnB) }
			if input.key_down(BTN_SELECT) { joypad.btn_down(GBInput::BtnSelect) } else { joypad.btn_up(GBInput::BtnSelect) }
			if input.key_down(BTN_START) { joypad.btn_down(GBInput::BtnStart) } else { joypad.btn_up(GBInput::BtnStart) }

			if input.key_down(DPAD_UP) { joypad.btn_down(GBInput::DPadUp) } else { joypad.btn_up(GBInput::DPadUp) }
			if input.key_down(DPAD_DOWN) { joypad.btn_down(GBInput::DPadDown) } else { joypad.btn_up(GBInput::DPadDown) }
			if input.key_down(DPAD_LEFT) { joypad.btn_down(GBInput::DPadLeft) } else { joypad.btn_up(GBInput::DPadLeft) }
			if input.key_down(DPAD_RIGHT) { joypad.btn_down(GBInput::DPadRight) } else { joypad.btn_up(GBInput::DPadRight) }

		});

		if !self.control.paused {

			'update:
			for i in 0..self.control.speed {
				self.emu.run_frame();

				for i in self.control.breakpoints.iter() {
					if self.emu.cpu.pc == *i {
						self.control.paused = true;
						break 'update;
					}
				}
			}

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