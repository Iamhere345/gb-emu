// Sample rate of 48Khz
pub const SAMPLE_RATE: usize = 48000;

// The size of the audio sample buffer.
pub const BUFFER_SIZE: usize = 1024;

// The rate at which the CPU is ticked.
pub const CPU_CLOCK: usize = 4194304;

// table of all wave duty values
const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

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

	channel_2: SquareChannel1,

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

			channel_2: SquareChannel1::default(),
		}
	}

	pub fn tick(&mut self, cycles: u64) {
		for _ in 0..cycles {

			self.sample_clock = self.sample_clock.wrapping_add(1);

			self.channel_2.tick();

			if self.sample_clock % 0x2000 == 0 {

				self.sample_clock = 0;

				match self.frame_sequencer_pos {
					0 => {
						self.channel_2.tick_length()
					},
					2 => {
						self.channel_2.tick_length()
					},
					4 => {
						self.channel_2.tick_length()
					},
					6 => {
						self.channel_2.tick_length()
					},
					7 => {
						self.channel_2.tick_volume();
					},
					_ => {}
				}

				// wraps around after 7
				self.frame_sequencer_pos = (self.frame_sequencer_pos + 1) % 8;
			}

			if self.sample_clock % ((CPU_CLOCK / SAMPLE_RATE) as u32) == 0 {
				self.buffer[self.buffer_pos] = (self.left_volume as f32 / 7.0)
					* (if (self.nr51 & 0x20) != 0 {
						self.channel_2.get_amplitude()
					} else {
						0.0
					} / 1.0);
				
				self.buffer[self.buffer_pos + 1] = (self.right_volume as f32 / 7.0)
				* (if (self.nr51 & 0x20) != 0 {
					self.channel_2.get_amplitude()
				} else {
					0.0
				} / 1.0);

				//println!("L: {} R: {}", self.buffer[self.buffer_pos], self.buffer[self.buffer_pos + 1]);

				if self.channel_2.get_amplitude() > -1.0 {
					//println!("ch2 amplitude: {}", self.channel_2.get_amplitude());
				}

				self.buffer_pos += 2;
			}

			if self.buffer_pos >= BUFFER_SIZE {

				//println!("pos: {}", self.buffer_pos);

				(self.callback)(self.buffer.as_ref());

				self.buffer_pos = 0;
			}

		}
	}

	pub fn read_byte(&self, addr: u16) -> u8 {
		match addr {
			// NR52: Audio Master Control
			0xFF26 => ((self.enabled as u8) << 7) | 0x70
				| (0)
				| (self.channel_2.enabled as u8) << 1
				| (0 << 2)
				| (0 << 3),
			// NR51: sound panning
			0xFF25 => self.nr51,
			// NR50: master volume & VIN panning
			0xFF24 => ((self.vin_left as u8) << 7)
				| (self.left_volume << 4)
				| ((self.vin_right as u8) << 3)
				| (self.right_volume),
			// Channel 1 IO
			0xFF10 ..= 0xFF14 => 0xFF,
			// Channel 2 IO
			0xFF16 ..= 0xFF19 => self.channel_2.read(addr),
			// Channel 3 IO + wave ram
			0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => 0xFF,
			// Channel 4 IO
			0xFF1F..=0xFF23 => 0xFF,
			_ => unreachable!()
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
			0xFF10 ..= 0xFF14 => {},
			// Channel 2 IO
			0xFF16 ..= 0xFF19 => self.channel_2.write(addr, write),
			// Channel 3 IO + wave ram
			0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => {},
			// Channel 4 IO
			0xFF1F..=0xFF23 => {},
			_ => unreachable!()
		}
	}

}

#[derive(Default)]
struct SquareChannel1 {

	enabled: bool,
	dac_enabled: bool,

	// index into WAVE_DUTY
	duty_pattern: usize,
	// index into WAVE_DUTY[self.wave_pattern]
	wave_position: usize,

	// decremented every T-state, increments wave_positoin when it reaches 0
	frequency_timer: u16,
	frequency: u16,

	initial_volume: u8,
	current_volume: u8,

	inc_volume: bool,
	envelope_period: u8,
	envelope_timer: u8,

	length_timer: u8,
	length_enabled: bool,
}

impl SquareChannel1 {

	pub fn tick(&mut self) {

		if self.frequency_timer == 0 {

			self.frequency_timer = (2048 - self.frequency) * 4;

			// wraps around after 7
			self.wave_position = (self.wave_position + 1) % 8;

		}

		self.frequency_timer -= 1;

	}

	pub fn tick_volume(&mut self) {
		if self.envelope_period != 0 {

			if self.envelope_timer > 0 {
				self.envelope_timer -= 1;
			}

			if self.envelope_timer == 0 {
				self.envelope_timer = self.envelope_period;

				if (self.current_volume < 0xF && self.inc_volume) || (self.current_volume > 0 && !self.inc_volume) {
					if self.inc_volume {
						self.current_volume += 1;
					} else {
						self.current_volume -= 1;
					}
				}
			}

		}
	}

	pub fn tick_length(&mut self) {
		if self.length_enabled && self.length_timer > 0 {
			self.length_timer -= 1;
			
			if self.length_timer == 0 {
				self.enabled = false;
			}
		}
	}

	pub fn get_amplitude(&self) -> f32 {
		if self.dac_enabled && self.enabled {

			//println!("duty: {}, vol: {}", WAVE_DUTY[self.duty_pattern][self.wave_position], self.current_volume);

			let dac_input = WAVE_DUTY[self.duty_pattern][self.wave_position] as f32 * self.current_volume as f32;

			(dac_input / 7.5) - 1.0
		} else {
			0.0
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			// there is no NR20
			0xFF15 => 0xFF,
			// NR21: duty patten & initial length timer (length is write only)
			0xFF16 => ((self.duty_pattern as u8) << 6) | 0b0011_1111,
			// NR22: initial volume & envelope
			0xFF17 => (self.initial_volume << 4) | ((self.inc_volume as u8) << 3) | self.envelope_period,
			// NR23: lower 8 bits of the frequency (write only)
			0xFF28 => 0xFF,
			// NR24: upper 3 bits of frequency (write only), trigger bit (write only), length timer enabled
			0xFF29 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

			_ => unreachable!()
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			// there is no NR20
			0xFF15 => {},
			// NR21: duty patten & initial length timer
			0xFF16 => {
				self.duty_pattern = ((write >> 6) & 0b11) as usize;
				self.length_timer = 64 - (write & 0b0011_1111);
			},
			// NR22: initial volume & envelope
			0xFF17 => {
				self.initial_volume = write >> 4;
				self.inc_volume = (write & 0x8) != 0;
				self.envelope_period = write & 0x7;

				self.dac_enabled = (write & 0b1111_1000) != 0;
			},
			// NR23: lower 8 bits of the frequency (write only)
			0xFF18 => {
				self.frequency = (self.frequency & 0x700) | write as u16;
			},
			// NR24: upper 3 bits of frequency, trigger bit, length timer enabled
			0xFF19 => {
				self.frequency = (self.frequency & 0xFF) | (((write & 0x7) as u16) << 8);

				self.length_enabled = ((write >> 6) & 0x1) != 0;

				if self.length_timer == 0 {
					self.length_timer = 64;
				}

				let trigger = (write >> 7) != 0;

				if trigger && self.dac_enabled {
					self.enabled = true;

					// reset envelope
					self.envelope_timer = self.envelope_period;
					self.current_volume = self.initial_volume;
				}

			},

			_ => unreachable!()
		}
	}

}