mod square;
mod wave;
mod noise;

use square::{SquareChannel1, SquareChannel2};
use wave::WaveChannel;
use noise::NoiseChannel;

// Sample rate of 48Khz
pub const SAMPLE_RATE: usize = 48000;

// The size of the audio sample buffer.
pub const BUFFER_SIZE: usize = 1024;

// The rate at which the CPU is ticked.
pub const CPU_CLOCK: usize = 4194304;

pub struct APU {

	enabled: bool,

	// controls sound panning
	nr51: u8,
	left_volume: u8,
	right_volume: u8,

	// stubbed - no commercial game ever used cartridge audio
	vin_left: bool,
	vin_right: bool,

	buffer: Box<[f32; BUFFER_SIZE]>,
	buffer_pos: usize,

	callback: Box<dyn Fn(&[f32])>,

	sample_clock: u32,

	frame_sequencer_pos: u8,

	channel_1: SquareChannel1,
	channel_2: SquareChannel2,
	channel_3: WaveChannel,
	channel_4: NoiseChannel,

}

impl APU {

	pub fn new(callback: Box<dyn Fn(&[f32])>) -> Self {
		Self {
			enabled: false,

			nr51: 0,
			left_volume: 0,
			right_volume: 0,

			vin_left: false,
			vin_right: false,

			buffer: Box::new([0.0; BUFFER_SIZE]),
			buffer_pos: 0,

			callback: callback,

			sample_clock: 0,
			frame_sequencer_pos: 0,

			channel_1: SquareChannel1::default(),
			channel_2: SquareChannel2::default(),
			channel_3: WaveChannel::default(),
			channel_4: NoiseChannel::default(),
		}
	}

	pub fn tick(&mut self, cycles: u64) {
		for _ in 0..cycles {

			self.sample_clock = self.sample_clock.wrapping_add(1);

			self.channel_1.tick();
			self.channel_2.tick();
			self.channel_3.tick();
			self.channel_4.tick();

			if self.sample_clock % 0x2000 == 0 {

				self.sample_clock = 0;

				match self.frame_sequencer_pos {
					0 => {
						self.channel_1.tick_length();
						self.channel_2.tick_length();
						self.channel_3.tick_length();
						self.channel_4.tick_length();
					},
					2 => {
						self.channel_1.tick_length();
						self.channel_2.tick_length();
						self.channel_3.tick_length();
						self.channel_4.tick_length();

						self.channel_1.tick_sweep();
					},
					4 => {
						self.channel_1.tick_length();
						self.channel_2.tick_length();
						self.channel_3.tick_length();
						self.channel_4.tick_length();
					},
					6 => {
						self.channel_1.tick_length();
						self.channel_2.tick_length();
						self.channel_3.tick_length();
						self.channel_4.tick_length();

						self.channel_1.tick_sweep();
					},
					7 => {
						self.channel_1.tick_volume();
						self.channel_2.tick_volume();
						self.channel_4.tick_volume();
					},
					_ => {}
				}

				// wraps around after 7
				self.frame_sequencer_pos = (self.frame_sequencer_pos + 1) % 8;
			}

			if self.sample_clock % ((CPU_CLOCK / SAMPLE_RATE) as u32) == 0 {
				self.buffer[self.buffer_pos] = (self.left_volume as f32 / 7.0)
					* ((if (self.nr51 & 0x10) != 0 {
						self.channel_1.get_amplitude()
					} else {
						0.0
					} + if (self.nr51 & 0x20) != 0 {
						self.channel_2.get_amplitude()
					} else {
						0.0
					} + if (self.nr51 & 0x40) != 0 {
						self.channel_3.get_amplitude()
					} else {
						0.0
					} + if (self.nr51 & 0x80) != 0 {
						self.channel_4.get_amplitude()
					} else {
						0.0
					}) / 4.0);
				
				self.buffer[self.buffer_pos + 1] = (self.right_volume as f32 / 7.0)
				* ((if (self.nr51 & 0x1) != 0 {
					self.channel_1.get_amplitude()
				} else {
					0.0
				} + if (self.nr51 & 0x2) != 0 {
					self.channel_2.get_amplitude()
				} else {
					0.0
				} + if (self.nr51 & 0x4) != 0 {
					self.channel_3.get_amplitude()
				} else {
					0.0
				} + if (self.nr51 & 0x8) != 0 {
					self.channel_4.get_amplitude()
				} else {
					0.0
				}) / 4.0);

				self.buffer_pos += 2;
			}

			if self.buffer_pos >= BUFFER_SIZE {
				(self.callback)(self.buffer.as_ref());

				self.buffer_pos = 0;
			}

		}
	}

	pub fn read_byte(&self, addr: u16) -> u8 {
		match addr {
			// NR52: Audio Master Control
			0xFF26 => ((self.enabled as u8) << 7) | 0x70
				| (self.channel_1.enabled as u8)
				| (self.channel_2.enabled as u8) << 1
				| (self.channel_3.enabled as u8) << 2
				| (self.channel_4.enabled as u8) << 3,
			// NR51: sound panning
			0xFF25 => self.nr51,
			// NR50: master volume & VIN panning
			0xFF24 => ((self.vin_left as u8) << 7)
				| (self.left_volume << 4)
				| ((self.vin_right as u8) << 3)
				| (self.right_volume),
			// Channel 1 IO
			0xFF10 ..= 0xFF14 => self.channel_1.read(addr),
			// Channel 2 IO
			0xFF16 ..= 0xFF19 => self.channel_2.read(addr),
			// Channel 3 IO + wave ram
			0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_3.read(addr),
			// Channel 4 IO
			0xFF1F..=0xFF23 => self.channel_4.read(addr),

			_ => panic!("0x{:X}", addr)
		}
	}

	pub fn write_byte(&mut self, addr: u16, write: u8) {
		match addr {
			// NR52: Audio Master Control
			0xFF26 => {
				// TODO clear registers/state when disabled
				self.enabled = (write & 0x80) != 0
			},
			// NR51: sound panning
			0xFF25 => self.nr51 = write,
			// NR50: master volume & VIN panning
			0xFF24 => {
				self.vin_left = (write & 0x80) != 0;
				self.vin_right = (write & 0x8) != 0;

				self.left_volume = (write >> 4) & 0x7;
				self.right_volume = write & 0x7;
			},
			// Channel 1 IO
			0xFF10 ..= 0xFF14 => self.channel_1.write(addr, write),
			// Channel 2 IO
			0xFF15 ..= 0xFF19 => self.channel_2.write(addr, write),
			// Channel 3 IO + wave ram
			0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_3.write(addr, write),
			// Channel 4 IO
			0xFF1F..=0xFF23 => self.channel_4.write(addr, write),

			_ => panic!("0x{:X}", addr)
		}
	}

}