#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Register8Bit {
	A,
	F,
	B,
	C,
	D,
	E,
	H,
	L,
	HL	// [HL] - only used as an identifier - cannot be accessed or modified by get_8bit_reg() or set_8bit_reg()
}

#[derive(PartialEq, Clone, Copy)]
pub enum Register16Bit {
	AF,
	BC,
	DE,
	HL,
	SP,
}

#[derive(Clone, Copy)]
pub enum Flag {
	Z = 7,	// zero
	N = 6,	// negative / subtraction
	H = 5,	// half-carry
	C = 4	// carry
}

#[allow(dead_code)]
pub struct Registers {
	// accumulator & flags (AF)
	a: u8,
	f: u8,

	// general purpose registers
	// BC
	b: u8,
	c: u8,
	// DE
	d: u8,
	e: u8,
	// HL
	h: u8,
	l: u8,

	// other registers
	sp: u16,    // stack pointer
}

impl Registers {
	pub fn new() -> Registers {
		Registers {
			// set to values for gameboy doctor
			a: 0x01,
			f: 0xB0,
			
			b: 0x00,
			c: 0x13,

			d: 0x00,
			e: 0xD8,

			h: 0x01,
			l: 0x4D,

			sp: 0xFFFE,
		}
	}
}

#[allow(dead_code)]
impl Registers {
	pub fn get_8bit_reg(&self, reg: Register8Bit) -> u8 {
		return match reg {
			Register8Bit::A => self.a,
			Register8Bit::F => self.f,
			Register8Bit::B => self.b,
			Register8Bit::C => self.c,
			Register8Bit::D => self.d,
			Register8Bit::E => self.e,
			Register8Bit::H => self.h,
			Register8Bit::L => self.l,
			_ => panic!("[HL] register enum cannot be accessed; don't read/write to it.")                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                
		};
	}

	pub fn set_8bit_reg(&mut self, reg_to_write: Register8Bit, set: u8) {
		let reg = match reg_to_write {
			Register8Bit::A => &mut self.a,
			Register8Bit::F => &mut self.f,
			Register8Bit::B => &mut self.b,
			Register8Bit::C => &mut self.c,
			Register8Bit::D => &mut self.d,
			Register8Bit::E => &mut self.e,
			Register8Bit::H => &mut self.h,
			Register8Bit::L => &mut self.l   ,
			_ => panic!("[HL] register enum cannot be modified; don't read/write to it.")                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              
		};

		*reg = set;
	}

	pub fn get_16bit_reg(&self, reg: Register16Bit) -> u16 {

		if reg == Register16Bit::SP {
			return self.sp
		}

		let (hi_reg, lo_reg): (u8, u8) = match reg {
			Register16Bit::AF => (self.a, self.f),
			Register16Bit::BC => (self.b, self.c),
			Register16Bit::DE => (self.d, self.e),
			Register16Bit::HL => (self.h, self.l),
			_ => panic!("invalid register")
		};

		(hi_reg as u16) << 8 | (lo_reg as u16)
	}

	pub fn set_16bit_reg(&mut self, reg: Register16Bit, write: u16) {

		let hi_reg: &mut u8;
		let lo_reg: &mut u8;

		match reg {
			Register16Bit::SP => {
				self.sp = write;
				return;
			},

			Register16Bit::AF => {
				hi_reg = &mut self.a;
				lo_reg = &mut self.f;
			},
			Register16Bit::BC => {
				hi_reg = &mut self.b;
				lo_reg = &mut self.c;
			},
			Register16Bit::DE => {
				hi_reg = &mut self.d;
				lo_reg = &mut self.e;
			},
			Register16Bit::HL => {
				hi_reg = &mut self.h;
				lo_reg = &mut self.l
			}
		}

		*hi_reg = ((write & 0xFF00) >> 8) as u8;
		*lo_reg = (write & 0xFF) as u8;
	}

	pub fn get_flag(&self, flag: Flag) -> bool {
		(self.f >> (flag as u8)) & 0b1 != 0
	}

	pub fn set_flag(&mut self, flag: Flag, set: bool) {
		self.f &= self.f & !(1 << (flag as u8));
		self.f |= (set as u8) << (flag as u8);
	}

}

impl Register16Bit {
	pub fn from_r16(r16: u8) -> Self {
		match r16 {
			0 => Self::BC,
			1 => Self::DE,
			2 => Self::HL,
			3 => Self::SP,
			_ => panic!("invalid r16")
		}
	}

	pub fn from_r16mem(r16mem: u8) -> (Self, i8) {
		match r16mem {
			0 => (Self::BC, 0),
			1 => (Self::DE, 0),
			2 => (Self::HL, 1),
			3 => (Self::HL, -1),
			_ => panic!("Invalid r16mem")
		}
	}
	pub fn from_r16stk(r16stk: u8) -> Self {
		match r16stk {
			0 => Self::BC,
			1 => Self::DE,
			2 => Self::HL,
			3 => Self::AF,
			_ => panic!("Invalid r16stk")
		}
	}
}

impl Register8Bit {
	pub fn from_r8(r8: u8) -> Self {
		match r8 {
			0 => Self::B,
			1 => Self::C,
			2 => Self::D,
			3 => Self::E,
			4 => Self::H,
			5 => Self::L,
			6 => Self::HL,
			7 => Self::A,
			_ => panic!("Invalid r8")
		}
	}
}