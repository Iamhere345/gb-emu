use std::time::Duration;

use eframe::{run_native, App, NativeOptions};
use app::Debugger;

mod app;
mod components;

fn main() {
	let native_options = NativeOptions::default();

    run_native("GB Emulator", native_options, Box::new(|cc| Box::new(Debugger::new(cc)))).expect("Unable to initialise egui app");

}