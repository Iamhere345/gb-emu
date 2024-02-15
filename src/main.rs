use gb::Gameboy;

mod gb;

fn main() {

	let mut gb: Gameboy = Gameboy::new();
	gb.init(include_bytes!("../tetris.gb"/*"../tests/cpu_instrs/individual/06-ld r,r.gb"*/));

}
