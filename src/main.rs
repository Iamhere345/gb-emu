use std::fs::File;
use std::io::prelude::*;
use std::iter;

use bus::*;
use cpu::cpu::CPU;

mod cpu;
mod bus;

fn main() {
    
	let bytes = include_bytes!("../test.gb");

	let mut bus: Bus = Bus::new();
	let mut cpu: CPU = CPU::new(&mut bus);

	for (addr, byte) in bytes.iter().enumerate() {
		cpu.bus.write_byte(addr.try_into().unwrap(), *byte);
	}

	cpu.pc = 0x0;

	for i in 0..10 {
		cpu.cycle();
	}

}
