use std::time::Duration;

use gb::Gameboy;

mod gb;

const CLOCK_CYCLE_SPEED: f64 = 1.0 / 4194304.0;

fn main() {

	let mut gb: Gameboy = Gameboy::new();
	gb.init(include_bytes!("../tetris.gb"));//"../tests/cpu_instrs/individual/06-ld r,r.gb"));

	// main loop
	loop {

		gb.tick();

		std::thread::sleep(Duration::from_secs_f64(CLOCK_CYCLE_SPEED));

	}

}
