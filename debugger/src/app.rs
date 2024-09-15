use eframe::{egui::{self, Key, Vec2}, App};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use gilrs::Gilrs;
use gilrs::ev::{Button, Axis};

use emu::Gameboy;
use emu::joypad::*;

use crate::components::{control::Control, cpu::Cpu, display::Display, ppu::Ppu, cart::Cart};

const BTN_A: Key 		= Key::Z;
const BTN_B: Key 		= Key::X;
const BTN_START: Key 	= Key::Enter;
const BTN_SELECT: Key 	= Key::Backspace;

const DPAD_UP: Key 		= Key::ArrowUp;
const DPAD_DOWN: Key 	= Key::ArrowDown;
const DPAD_LEFT: Key 	= Key::ArrowLeft;
const DPAD_RIGHT: Key 	= Key::ArrowRight;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

#[derive(PartialEq)]
enum DpadDir {
	Up,
	Down,
	Left,
	Right,
	None
}

pub struct Debugger {
	emu: Gameboy,

	_audio_stream: OutputStream,
	stream_handle: OutputStreamHandle,

	gilrs: Gilrs,

	display: Display,

	control: Control,
	cpu: Cpu,
	ppu: Ppu,
	cart: Cart,

	debug_mode: bool,
	just_changed_mode: bool,

	window_scale: f32,

	show_help: bool,
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

			_audio_stream: stream,
			stream_handle: stream_handle,

			gilrs: Gilrs::new().unwrap(),
			
			display: Display::new(cc),

			control: Control::new(),
			cpu: Cpu::new(),
			ppu: Ppu::new(),
			cart: Cart::new(),

			debug_mode: false,
			just_changed_mode: true,

			window_scale: 6.0,

			show_help: false,
		}
	}

	fn is_gamepad_input_down(&mut self, button: &Button, dir: DpadDir) -> bool {
		let left_stick = self.gilrs.gamepads().any(|(_, g)| {
            if let (Some(axis_x), Some(axis_y)) = (g.axis_data(Axis::LeftStickX), g.axis_data(Axis::LeftStickY)) {
                axis_x.value() > 0.5 && dir == DpadDir::Right
                    || axis_x.value() < -0.5 && dir == DpadDir::Left
                    || axis_y.value() > 0.5 && dir == DpadDir::Up
                    || axis_y.value() < -0.5 && dir == DpadDir::Down
            } else {
                false
            }
        });

		let btn_press = self.gilrs.gamepads().any(|(_, gamepad)| {
			gamepad.is_pressed(*button)
		});

		self.gilrs.next_event();

		left_stick || btn_press
	}

	fn is_keyboard_input_down(&mut self, key: Key, ctx: &egui::Context) -> bool {
		ctx.input(|input| {
			input.key_down(key)
		})
	}

	fn handle_input(&mut self, ctx: &egui::Context) {

		let gp_dpad_left = self.is_gamepad_input_down(&Button::DPadLeft, DpadDir::Left);
		let kb_dpad_left = self.is_keyboard_input_down(DPAD_LEFT, ctx);

		let gp_dpad_right = self.is_gamepad_input_down(&Button::DPadRight, DpadDir::Right);
		let kb_dpad_right = self.is_keyboard_input_down(DPAD_RIGHT, ctx);

		let gp_dpad_up = self.is_gamepad_input_down(&Button::DPadUp, DpadDir::Up);
		let kb_dpad_up = self.is_keyboard_input_down(DPAD_UP, ctx);

		let gp_dpad_down = self.is_gamepad_input_down(&Button::DPadDown, DpadDir::Down);
		let kb_dpad_down = self.is_keyboard_input_down(DPAD_DOWN, ctx);

		if gp_dpad_left || (kb_dpad_left && !kb_dpad_right) { self.emu.btn_down(GBInput::DPadLeft) } else { self.emu.btn_up(GBInput::DPadLeft) }
		if gp_dpad_right || (kb_dpad_right && !kb_dpad_left) { self.emu.btn_down(GBInput::DPadRight) } else { self.emu.btn_up(GBInput::DPadRight) }
		if gp_dpad_up || (kb_dpad_up && !kb_dpad_down) { self.emu.btn_down(GBInput::DPadUp) } else { self.emu.btn_up(GBInput::DPadUp) }
		if gp_dpad_down || (kb_dpad_down && !kb_dpad_up) { self.emu.btn_down(GBInput::DPadDown) } else { self.emu.btn_up(GBInput::DPadDown) }

		if self.is_gamepad_input_down(&Button::South, DpadDir::None) || self.is_keyboard_input_down(BTN_A, ctx) { self.emu.btn_down(GBInput::BtnA) } else { self.emu.btn_up(GBInput::BtnA) }
		if self.is_gamepad_input_down(&Button::East, DpadDir::None) || self.is_keyboard_input_down(BTN_B, ctx) { self.emu.btn_down(GBInput::BtnB) } else { self.emu.btn_up(GBInput::BtnB) }
		if self.is_gamepad_input_down(&Button::Start, DpadDir::None) || self.is_keyboard_input_down(BTN_START, ctx) { self.emu.btn_down(GBInput::BtnStart) } else { self.emu.btn_up(GBInput::BtnStart) }
		if self.is_gamepad_input_down(&Button::Select, DpadDir::None) || self.is_keyboard_input_down(BTN_SELECT, ctx) { self.emu.btn_down(GBInput::BtnSelect) } else { self.emu.btn_up(GBInput::BtnSelect) }

		ctx.input(|input| {
			if input.key_pressed(Key::Escape) {
				self.debug_mode = !self.debug_mode;
				self.just_changed_mode = true;
			}
		})
		
	}
}

impl App for Debugger {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		
		self.handle_input(ctx);

		if self.just_changed_mode {
			self.just_changed_mode = false;

			if self.debug_mode {
				ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(1280.0, 720.0)));
			} else {
				ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
					Vec2::new(SCREEN_WIDTH as f32 * self.window_scale, SCREEN_HEIGHT as f32 * self.window_scale))
				);
			}
		}

		if !self.control.paused {

			'update:
			while !self.emu.tick() {
				for i in self.control.breakpoints.iter() {
					if self.emu.cpu.pc == *i {
						self.control.paused = true;
						break 'update;
					}
				}
			}

		}

		if self.debug_mode {
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

		} else {

			egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
				ui.horizontal(|ui| {
					
					self.control.show_start_speed(ui, &mut self.emu);
					
					if ui.button(format!("Scale: {}x", self.window_scale)).clicked() {
						self.window_scale = match self.window_scale as u32 {
							1 => 2.0,
							2 => 4.0,
							4 => 6.0,
							_ => 1.0
						};
						
						ctx.send_viewport_cmd(
							egui::ViewportCommand::InnerSize(Vec2::new(SCREEN_WIDTH as f32 * self.window_scale, SCREEN_HEIGHT as f32 * self.window_scale))
						);
						
						
					}

					self.control.show_select_rom(ui, &mut self.emu, &mut self.stream_handle);

					if ui.button("Help").clicked() {
						self.show_help = !self.show_help;
					}

					if self.show_help {
						egui::Window::new("Help").show(ctx, |ui| {
							ui.heading("Controls");

							ui.separator();

							ui.strong("Keyboard");

							ui.label("Z: A");
							ui.label("X: B");
							ui.label("Arrow keys: Dpad");
							ui.label("Enter: Start");
							ui.label("Backspace: Select");

							ui.separator();

							ui.strong("Controller");

							ui.label("South / X / A: A");
							ui.label("East / O / B: B");
							ui.label("Left stick / Dpad: Dpad");
							ui.label("Start: Start");
							ui.label("Select / Share: Select");

						});
					}

					ui.checkbox(&mut self.control.enable_bootrom, "Enable Bootrom");

				});

			});

		}

		egui::CentralPanel::default().show(ctx, |ui| {
			self.display.show(ctx, ui, &mut self.emu, self.control.scale, self.debug_mode)
		});

		ctx.request_repaint();
		
	}
}

impl Drop for Debugger {
	fn drop(&mut self) {
		self.control.save_sram(&self.emu);
	}
}