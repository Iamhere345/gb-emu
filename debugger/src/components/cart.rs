use eframe::egui::*;

use emu::Gameboy;

pub struct Cart {
	pub enable_bootrom: bool,
}

impl Cart {

	pub fn new() -> Self {
		Cart {
			enable_bootrom: false,
		}
	}

	pub fn show(&mut self, _ctx: &Context, ui: &mut Ui, emu: &mut Gameboy) {

		ui.strong("Cartridge");

		let cart_id = emu.bus.borrow().read_byte(0x147);

		let cart_type = match cart_id {
			0x0 => "ROM ONLY".to_string(),
			0x1 => "MBC1".to_string(),
			0x2 => "MBC1+RAM".to_string(),
			0x3 => "MBC1+RAM+BATTERY".to_string(),
			0x5 => "MBC2".to_string(),
			0x6 => "MBC2+BATTERY".to_string(),
			0x8 => "ROM+RAM".to_string(),
			0x9 => "ROM+RAM+BATTERY".to_string(),
			0xB => "MMM01".to_string(),
			0xC => "MMM01+RAM".to_string(),
			0xD => "MMM01+RAM+BATTERY".to_string(),
			0xF => "MBC3+TIMER+BATTERY".to_string(),
			0x10 => "MBC3+TIMER+RAM+BATTERY".to_string(),
			0x11 => "MBC3".to_string(),
			0x12 => "MBC3+RAM".to_string(),
			0x13 => "MBC3+RAM+BATTERY".to_string(),
			0x19 => "MBC5".to_string(),
			0x1A => "MBC5+RAM".to_string(),
			0x1B => "MBC5+RAM+BATTERY".to_string(),
			0x1C => "MBC5+RUMBLE".to_string(),
			0x1D => "MBC5+RUMBLE+RAM".to_string(),
			0x1E => "MBC5+RUMBLE+RAM+BATTERY".to_string(),
			0x20 => "MBC6".to_string(),
			0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY".to_string(),
			0xFC => "POCKET CAMERA".to_string(),
			0xFD => "BANDAI TAMA5".to_string(),
			0xFE => "HuC3".to_string(),
			0xFF => "HuC1+RAM+BATTERY".to_string(),
			_ => format!("UNKNOWN (0x{:X})", cart_id),
		};

		ui.monospace(format!("MBC Type: {cart_type}"));

		ui.monospace(format!("ROM Banks: {}", 2 * (2 as usize).pow(emu.bus.borrow().read_byte(0x148) as u32)));

		let ram_size = match emu.bus.borrow().read_byte(0x149) {
			2 			=> 8,
			3 			=> 32,
			4 			=> 128,
			5			=> 64,
			0 | 1 | _ 	=> 0,
		};

		ui.monospace(format!("RAM Size: {} KiB", ram_size));

		let cgb_support = match emu.bus.borrow().read_byte(0x143) {
			0x80 => "Enhanced",
			0xC0 => "Exclusive",
			_ => "N/A"
		};
		
		ui.monospace(format!("CGB: {cgb_support}"));
		ui.monospace(format!("SGB: {}", if emu.bus.borrow().read_byte(0x146) == 0x03 { "Enhanced" } else { "N/A" }));
		

	}

}