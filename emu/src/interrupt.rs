// used for setting bits in IE and IF
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterruptFlag {
	VBlank = 1 << 0,
	LCDC = 1 << 1,
	Timer = 1 << 2,
	Serial = 1 << 3,
	Joypad = 1 << 4,
}

impl InterruptFlag {
	pub fn from_u8(from: u8) -> Self {
		match from {
			1 => Self::VBlank,
			2 => Self::LCDC,
			4 => Self::Timer,
			8 => Self::Serial,
			16 => Self::Joypad,
			_ => panic!("invalid interrupt flag")
		}
	}
}

pub enum InterruptSource {
	VBlank = 0x40,
	LCDC = 0x48,
	Timer = 0x50,
	Serial = 0x58,
	Joypad = 0x60
}

impl InterruptSource {
	pub fn from_flag(from: InterruptFlag) -> Self {
		match from {
			InterruptFlag::VBlank => Self::VBlank,
			InterruptFlag::LCDC => Self::LCDC,
			InterruptFlag::Timer => Self::Timer,
			InterruptFlag::Serial => Self::Serial,
			InterruptFlag::Joypad => Self::Joypad
		}
	}
}

#[derive(Default)]
pub struct Interrupt {
	enable: u8,		// IE
	flags: u8		// IF
}

impl Interrupt {

	pub fn read(&self, addr: u16) -> u8 {

		match addr {
			0xFFFF => self.enable,
			0xFF0F => self.flags,
			_ => panic!("Invalid address")
		}

	}

	pub fn write(&mut self, addr: u16, write: u8) {

		match addr {
			0xFFFF => self.enable = write,
			0xFF0F => self.flags = write,
			_ => panic!("Invalid address")
		}

	}

	pub fn raise(&mut self, flag: InterruptFlag) {
		self.flags |= flag as u8;
	}

	pub fn clear(&mut self, flag: InterruptFlag) {
		self.flags &= !(flag as u8);
	}

	pub fn is_raised(&self, flag: InterruptFlag) -> bool {
		if self.flags & flag as u8 != 0 {
			return true;
		}
		false
	}

}