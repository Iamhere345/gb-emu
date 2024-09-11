use eframe::{egui::{self, Key}, App};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use emu::Gameboy;
use emu::joypad::*;

use crate::components::{control::Control, cpu::Cpu, display::Display, ppu::Ppu, cart::Cart};

//const CYCLES_PER_FRAME: usize = (4194304.0 / 60.0) as usize;
const CYCLES_PER_FRAME: u64 = 69905;

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

	audio_stream: OutputStream,
	stream_handle: OutputStreamHandle,

	display: Display,

	control: Control,
	cpu: Cpu,
	ppu: Ppu,
	cart: Cart,
}

impl Debugger {
	pub fn new(cc: &eframe::CreationContext) -> Self {

		let cart = std::fs::read("roms/dmg-acid2.gb").unwrap();

		let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

		let emu = Gameboy::new(cart, Box::new(move |buffer| {
			while sink.len() > 2 {
				std::thread::sleep(std::time::Duration::from_millis(1))
			}

			sink.append(SamplesBuffer::new(2, 48000, buffer));
		}));

		Self {
			emu: emu,

			audio_stream: stream,
			stream_handle: stream_handle,

			display: Display::new(cc),

			control: Control::new(),
			cpu: Cpu::new(),
			ppu: Ppu::new(),
			cart: Cart::new()
		}
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		
		ctx.input(|input| {

			let joypad = &mut self.emu.bus.borrow_mut().joypad;

			if input.key_down(BTN_A) { joypad.btn_down(GBInput::BtnA) } else { joypad.btn_up(GBInput::BtnA) }
			if input.key_down(BTN_B) { joypad.btn_down(GBInput::BtnB) } else { joypad.btn_up(GBInput::BtnB) }
			if input.key_down(BTN_SELECT) { joypad.btn_down(GBInput::BtnSelect) } else { joypad.btn_up(GBInput::BtnSelect) }
			if input.key_down(BTN_START) { joypad.btn_down(GBInput::BtnStart) } else { joypad.btn_up(GBInput::BtnStart) }

			if !(input.key_down(DPAD_LEFT) && input.key_down(DPAD_RIGHT)) {
				if input.key_down(DPAD_LEFT) { joypad.btn_down(GBInput::DPadLeft) } else { joypad.btn_up(GBInput::DPadLeft) }
				if input.key_down(DPAD_RIGHT) { joypad.btn_down(GBInput::DPadRight) } else { joypad.btn_up(GBInput::DPadRight) }
			} else {
				joypad.btn_up(GBInput::DPadLeft);
				joypad.btn_up(GBInput::DPadRight);
			}

			if !(input.key_down(DPAD_UP) && input.key_down(DPAD_DOWN)) {
				if input.key_down(DPAD_UP) { joypad.btn_down(GBInput::DPadUp) } else { joypad.btn_up(GBInput::DPadUp) }
				if input.key_down(DPAD_DOWN) { joypad.btn_down(GBInput::DPadDown) } else { joypad.btn_up(GBInput::DPadDown) }
			} else {
				joypad.btn_up(GBInput::DPadUp);
				joypad.btn_up(GBInput::DPadDown);
			}
			

		});

		if !self.control.paused {

			'update:
			for _ in 0..self.control.speed {

				loop {

					for i in self.control.breakpoints.iter() {
						if self.emu.cpu.pc == *i {
							self.control.paused = true;
							break 'update;
						}
					}

					let cycles = self.emu.tick();

					self.emu.cycles += cycles;
		
					if self.emu.cycles >= CYCLES_PER_FRAME {
						self.emu.cycles -= CYCLES_PER_FRAME;
						break;
					}
		
				}

				
			}

		}

		egui::SidePanel::left("left_pannel").show(ctx, |ui| {
			
			ui.heading("gb-emu");
			
			ui.separator();
			
			self.control.show(ctx, ui, &mut self.emu, &mut self.stream_handle);
			
			ui.separator();
			
			self.cpu.show(ctx, ui, &mut self.emu);
			
			ui.separator();
			
			self.ppu.show(ctx, ui, &mut self.emu);

			ui.separator();

			self.cart.show(ctx, ui, &mut self.emu);

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

impl Drop for Debugger {
	fn drop(&mut self) {
		self.control.save_sram(&self.emu);
	}
}