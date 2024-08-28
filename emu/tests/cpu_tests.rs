use std::fs;

use emu::{ cpu::{registers::*, *}, Gameboy};
use serde::{Deserialize, Serialize};

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

	let mut gb = Gameboy::new(Vec::new());

	for (i, entry_res) in fs::read_dir("../tests/sm83-test-data").expect("TEST ERROR: unable to read test data").enumerate() {

		let entry = entry_res.unwrap();

		/* if i < 21 {
			continue;
		} */

		let test_str: String = fs::read_to_string(entry.path()).unwrap();

		let test_data: Vec<Test> = serde_json::from_str(test_str.as_str()).expect(format!("Unable to parse test data in {}", entry.file_name().into_string().unwrap()).as_str());

		for test in test_data.iter() {

			println!("--------Test: {} ({})", test.name, i);

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
			gb.cpu.ei = 0;
			gb.cpu.halted = false;
			//gb.bus.borrow_mut().write_register(MemRegister::IE, test.initial_state.ie);

			/*
			for i in 0..test.cycles.len() {
				gb.tick();
			}
			*/
			

			gb.cpu.cycle();

			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::A), test.final_state.a, "A comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.a,
				test.final_state.a,
				gb.cpu.registers.get_8bit_reg(Register8Bit::A)	
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::B), test.final_state.b, "B comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.b,
				test.final_state.b,
				gb.cpu.registers.get_8bit_reg(Register8Bit::B)	
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::C), test.final_state.c, "C comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.c,
				test.final_state.c,
				gb.cpu.registers.get_8bit_reg(Register8Bit::C)	
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::D), test.final_state.d, "D comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.d,
				test.final_state.d,
				gb.cpu.registers.get_8bit_reg(Register8Bit::D)	
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::E), test.final_state.e, "E comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.e,
				test.final_state.e,
				gb.cpu.registers.get_8bit_reg(Register8Bit::E)	
			);

			let initial_test_flags = {
				let mut regs = Registers::new();
				regs.set_8bit_reg(Register8Bit::F, test.initial_state.f);

				format!("{}{}{}{}",
					if regs.get_flag(Flag::Z) { "Z" } else { "_" },
					if regs.get_flag(Flag::N) { "N" } else { "_" },
					if regs.get_flag(Flag::H) { "H" } else { "_" },
					if regs.get_flag(Flag::C) { "C" } else { "_" },
				)

			};

			let final_test_flags = {
				let mut regs = Registers::new();
				regs.set_8bit_reg(Register8Bit::F, test.final_state.f);

				format!("{}{}{}{}",
					if regs.get_flag(Flag::Z) { "Z" } else { "_" },
					if regs.get_flag(Flag::N) { "N" } else { "_" },
					if regs.get_flag(Flag::H) { "H" } else { "_" },
					if regs.get_flag(Flag::C) { "C" } else { "_" },
				)

			};

			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::F), test.final_state.f, "F comparison failed (initial: {} final: {} actual: {}{}{}{}) final 0x{:x} actual 0x{:x}",
				initial_test_flags,
				final_test_flags,
				if gb.cpu.registers.get_flag(Flag::Z) { "Z" } else { "_" },
				if gb.cpu.registers.get_flag(Flag::N) { "N" } else { "_" },
				if gb.cpu.registers.get_flag(Flag::H) { "H" } else { "_" },
				if gb.cpu.registers.get_flag(Flag::C) { "C" } else { "_" },
				test.final_state.f,
				gb.cpu.get_8bit_reg(Register8Bit::F),
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::H), test.final_state.h, "H comparison failed (initial: 0x{:x} final: 0x{:x} actual: 0x{:x})",
				test.initial_state.h,
				test.final_state.h,
				gb.cpu.registers.get_8bit_reg(Register8Bit::H),
			);
			assert_eq!(gb.cpu.registers.get_8bit_reg(Register8Bit::L), test.final_state.l, "L comparison failed (final 0x{:x} actual: 0x{:x})",
				test.final_state.h,
				gb.cpu.registers.get_8bit_reg(Register8Bit::L),
			);

			assert_eq!(gb.cpu.pc, test.final_state.pc, "PC comparison failed (initial 0x{:x} final 0x{:x} actual: 0x{:x})",
				test.initial_state.pc,
				test.final_state.pc,
				gb.cpu.pc,
			);
			assert_eq!(gb.cpu.registers.get_16bit_reg(Register16Bit::SP), test.final_state.sp, "SP comparison failed (final: 0x{:x} actual 0x{:x})",
				test.final_state.sp,
				gb.cpu.registers.get_16bit_reg(Register16Bit::SP),
			);
			assert_eq!(gb.cpu.ime, test.final_state.ime != 0, "IME comparison failed (initial: {} final: {} actual: {})",
				test.initial_state.ime != 0,
				test.final_state.ime != 0,
				gb.cpu.ime,
			);

			for (addr, data) in test.final_state.ram.iter() {
				assert_eq!(gb.bus.borrow().read_byte(*addr), *data, "[0x{:x}] RAM comparison failed (final: 0x{:x} actual: 0x{:x})", 
					*addr,
					*data, 
					gb.bus.borrow().read_byte(*addr)
				);
			}

			gb.bus.borrow_mut().clear_test_mem();

		}

	}

}