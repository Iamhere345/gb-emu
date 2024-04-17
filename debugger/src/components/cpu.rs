use std::fmt::format;

use eframe::egui::*;

use emu::{cpu, Gameboy};

pub struct Cpu;

impl Cpu {

	pub fn new() -> Self {
		Cpu {}
	}

	pub fn show(&mut self, ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		ui.label("CPU");

		ui.horizontal(|ui| {

			ui.label(format!("AF: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::AF)));
			ui.label(format!("BC: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::BC)));
			ui.label(format!("DE: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::DE)));
			ui.label(format!("HL: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::HL)));

		});

		ui.horizontal(|ui| {

			ui.label(format!("SP: 0x{:X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::SP)));
			ui.label(format!("PC: 0x{:X}", emu.cpu.pc));

			ui.label(format!("[HL]: 0x{:X}", emu.cpu.get_8bit_reg(emu::cpu::registers::Register8Bit::HL)));

			ui.label(format!("Cycles: {}", emu.cycles));0

		});

		ui.horizontal(|ui| {

			ui.label(format!("Flags: {}{}{}{}", 
				if emu.cpu.registers.get_flag(cpu::registers::Flag::Z) { "Z" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::N) { "N" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::H) { "H" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::C) { "C" } else { "_" },
			))

		});

		ui.label(format!("Executed Instruction Instruction: {}", emu.cpu.last_instruction));

	}

}