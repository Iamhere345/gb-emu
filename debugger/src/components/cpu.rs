use eframe::egui::*;

use emu::{cpu, Gameboy};

pub struct Cpu;

impl Cpu {

	pub fn new() -> Self {
		Cpu {}
	}

	pub fn show(&mut self, _ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		ui.strong("CPU");

		ui.horizontal(|ui| {

			ui.monospace(format!("AF: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::AF)));
			ui.monospace(format!("BC: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::BC)));
			ui.monospace(format!("DE: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::DE)));
			ui.monospace(format!("HL: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::HL)));

		});

		ui.horizontal(|ui| {

			ui.monospace(format!("SP: 0x{:04X}", emu.cpu.registers.get_16bit_reg(emu::cpu::registers::Register16Bit::SP)));
			ui.monospace(format!("PC: 0x{:04X}", emu.cpu.pc));

			ui.monospace(format!("[HL]: 0x{:X}", emu.cpu.get_8bit_reg(emu::cpu::registers::Register8Bit::HL)));

			ui.monospace(format!("Cycles: {}", emu.cycles));

		});

		ui.horizontal(|ui| {

			ui.monospace(format!("IME: {}", emu.cpu.ime));
			ui.monospace(format!("HALT: {}", emu.cpu.halted));

		});

		ui.horizontal(|ui| {

			ui.monospace(format!("Flags: {}{}{}{}", 
				if emu.cpu.registers.get_flag(cpu::registers::Flag::Z) { "Z" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::N) { "N" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::H) { "H" } else { "_" },
				if emu.cpu.registers.get_flag(cpu::registers::Flag::C) { "C" } else { "_" },
			))

		});

		ui.monospace(format!("Executed Instruction: {}", emu.cpu.last_instruction));

	}

}