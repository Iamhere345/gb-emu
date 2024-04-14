use std::time::Duration;

use eframe::{run_native, App, NativeOptions};
use app::Debugger;

mod app;
mod components;

fn main() {
	let native_options = NativeOptions::default();

    //run_native("GB Emulator", native_options, Box::new(|cc| Box::new(Debugger::new(cc)))).expect("Unable to initialise egui app");

	let mut emu = emu::Gameboy::new();

	emu.init(include_bytes!("../../tests/cpu_instrs/individual/02-interrupts.gb"));

	loop {
		let clock_cycles_per_frame: usize = (4194304.0 / 60.0) as usize;

		for _ in 0..clock_cycles_per_frame {
			emu.tick();
		}

		std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
	}
}