use std::{default, fs};

use emu::{bus::*, Gameboy, cpu::*};
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
struct InitialState {
	pc: u16,
	sp: u16,
	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	f: u8,
	h: u8,
	l: u8,
	ime: u8,
	ie: u8,
	ram: Vec<(u16, u8)>
}

#[derive(Serialize, Deserialize, Debug)]
struct FinalState {
	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	f: u8,
	h: u8,
	l: u8,
	pc: u16,
	sp: u16,
	ime: u8,
	ram: Vec<(u16, u8)>
}

#[derive(Serialize, Deserialize, Debug)]
struct Test {
	name: String,

	#[serde(alias = "initial")]
	initial_state: InitialState,

	#[serde(alias = "final")] 
	final_state: FinalState,

	cycles: Vec<(u16, u8, String)>,

}

// sm83-test-data from: https://github.com/raddad772/jsmoo-json-tests
#[test]
fn sm83_test_data() {

	let mut gb = Gameboy::new();

	for entry_res in fs::read_dir("../tests/sm83-test-data").expect("TEST ERROR: unable to read test data") {
		
		let entry = entry_res.unwrap();

		let test_str: String = fs::read_to_string(entry.path()).unwrap();

		let test_data: Vec<Test> = serde_json::from_str(test_str.as_str()).expect(format!("Unable to parse test data in {}", entry.file_name().into_string().unwrap()).as_str());

		for test in test_data.iter() {

			println!("Test: {}", test.name);

			for (addr, data) in test.initial_state.ram.iter() {
				gb.bus.borrow_mut().write_byte(*addr, *data);
			}

			gb.cpu.pc = test.initial_state.pc;
			gb.cpu.registers.set_16bit_reg(registers::Register16Bit::SP, test.initial_state.sp);

			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::A, test.initial_state.a);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::B, test.initial_state.b);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::C, test.initial_state.c);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::D, test.initial_state.d);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::E, test.initial_state.e);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::F, test.initial_state.f);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::H, test.initial_state.h);
			gb.cpu.registers.set_8bit_reg(registers::Register8Bit::L, test.initial_state.l);

			gb.cpu.ime = test.initial_state.ime != 0;
			gb.bus.borrow_mut().write_register(MemRegister::IE, test.initial_state.ie);

			for i in 0..test.cycles.len() {
				gb.tick();
			}

			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::A), test.final_state.a, "A comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::B), test.final_state.b, "B comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::C), test.final_state.c, "C comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::D), test.final_state.d, "D comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::E), test.final_state.e, "E comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::F), test.final_state.f, "F comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::H), test.final_state.h, "H comparison failed");
			assert_eq!(gb.cpu.registers.get_8bit_reg(registers::Register8Bit::L), test.final_state.l, "L comparison failed");

			assert_eq!(gb.cpu.pc, test.final_state.pc, "PC comparison failed");
			assert_eq!(gb.cpu.registers.get_16bit_reg(registers::Register16Bit::SP), test.final_state.sp, "SP comparison failed");
			assert_eq!(gb.cpu.ime, test.final_state.ime != 0, "IME comparison failed");

			for (addr, data) in test.final_state.ram.iter() {
				assert_eq!(gb.bus.borrow().read_byte(*addr), *data, "RAM comparison failed")
			}

		}

	}

}