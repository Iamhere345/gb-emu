use eframe::{run_native, NativeOptions};
use app::Debugger;

mod app;
mod components;

fn main() {
	let native_options = NativeOptions {
		vsync: true,
		..Default::default()
	};

    run_native("GB Emulator", native_options, Box::new(|cc| Box::new(Debugger::new(cc)))).expect("Unable to initialise egui app");

}