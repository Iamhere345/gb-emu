use std::time::Duration;
use std::io::{prelude::*, BufWriter};
use std::fs;

use gb::Gameboy;

mod gb;

const CLOCK_CYCLE_SPEED: f64 = 1.0 / (4194304.0 * 1_000_000_000.0);

fn main() {

	//../tests/cpu_instrs/individual/06-ld r,r.gb
	//../tetris.gb

	let mut gb: Gameboy = Gameboy::new();
	gb.init(include_bytes!("../tests/cpu_instrs/individual/06-ld r,r.gb"));

	//let log = File::create("emu.log").expect("unable to open log file");
	//let mut log_writer = BufWriter::new(&log);
	let log = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open("emu.log")
		.expect("unable to open or create log file");

    let mut log_writer = BufWriter::new(log);

	// main loop
	loop {

		gb.tick(&mut log_writer);

		//write!(&mut log_writer, "{}", log_line).expect("unable to write to log file");

		std::thread::sleep(Duration::from_secs_f64(CLOCK_CYCLE_SPEED));

	}

}
