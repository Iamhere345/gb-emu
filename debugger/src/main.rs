use eframe::{egui::{Vec2, ViewportBuilder}, run_native, NativeOptions};
use app::Debugger;

mod app;
mod components;

fn main() {

	let viewport = ViewportBuilder {
		inner_size: Some(Vec2::new(1280.0, 720.0)),
		..Default::default()
	};

	let native_options = NativeOptions {
		viewport: viewport,
		vsync: false,
		..Default::default()
	};

    run_native("GB Emulator", native_options, Box::new(|cc| Box::new(Debugger::new(cc)))).expect("Unable to initialise egui app");

}