// table of all wave duty values
const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

#[derive(Default)]
pub struct SquareChannel1 {

	pub enabled: bool,
	dac_enabled: bool,

	// index into WAVE_DUTY
	duty_pattern: usize,
	// index into WAVE_DUTY[self.wave_pattern]
	pub wave_position: usize,

	// decremented every T-state, increments wave_positoin when it reaches 0
	frequency_timer: u16,
	frequency: u16,

	initial_volume: u8,
	current_volume: u8,

	inc_volume: bool,
	envelope_period: u8,
	envelope_timer: u8,

	length_timer: u16,
	length_enabled: bool,

	sweep_enabled: bool,
	dec_freq: bool,
	sweep_period: u8,
	sweep_timer: u8,
	sweep_amount: u8,
	old_freq: u16,

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

	pub fn tick_sweep(&mut self) {
		if self.sweep_timer > 0 {
			self.sweep_timer -= 1;
		}

		if self.sweep_timer == 0 {
			if self.sweep_period > 0 {
				self.sweep_timer = self.sweep_period;
			} else {
				self.sweep_timer = 8;
			}

			if self.sweep_enabled && self.sweep_period > 0 {
				let new_freq = self.calc_frequency();

				if new_freq <= 2047 && self.sweep_amount > 0 {
					self.frequency = new_freq;
					self.old_freq = new_freq;

					self.calc_frequency();
				}
			}
		}
	}

	fn calc_frequency(&mut self) -> u16 {

		let mut new_freq = self.old_freq >> self.sweep_amount;

		if self.dec_freq {
			new_freq = self.old_freq - new_freq;
		} else {
			new_freq = self.old_freq + new_freq;
		}

		if new_freq > 2047 {
			self.enabled = false
		}

		new_freq

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
			let dac_input = WAVE_DUTY[self.duty_pattern][self.wave_position] as f32 * self.current_volume as f32;

			(dac_input / 7.5) - 1.0
		} else {
			0.0
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			// NR10: sweep register
			0xFF10 => (self.sweep_period << 4)
				| (if self.dec_freq { 0x8 } else { 0x0 })
				| self.sweep_amount
				| 0x80,
			// NR11: duty patten & initial length timer (length is write only)
			0xFF11 => ((self.duty_pattern as u8) << 6) | 0b0011_1111,
			// NR12: initial volume & envelope
			0xFF12 => (self.initial_volume << 4) | ((self.inc_volume as u8) << 3) | self.envelope_period,
			// NR13: lower 8 bits of the frequency (write only)
			0xFF13 => 0xFF,
			// NR14: upper 3 bits of frequency (write only), trigger bit (write only), length timer enabled
			0xFF14 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

			_ => unreachable!()
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			// NR10: sweep register
			0xFF10 => {
				self.dec_freq = (write & 0x8) != 0;
                self.sweep_period = write >> 4;
                self.sweep_amount = write & 0x7;
			},
			// NR11: duty patten & initial length timer
			0xFF11 => {
				self.duty_pattern = ((write >> 6) & 0b11) as usize;
				self.length_timer = 64 - (write & 0b0011_1111) as u16;
			},
			// NR12: initial volume & envelope
			0xFF12 => {
				self.initial_volume = write >> 4;
				self.inc_volume = (write & 0x8) != 0;
				self.envelope_period = write & 0x7;

				self.dac_enabled = (write & 0b1111_1000) != 0;
			},
			// NR13: lower 8 bits of the frequency (write only)
			0xFF13 => {
				self.frequency = (self.frequency & 0x700) | write as u16;
			},
			// NR14: upper 3 bits of frequency, trigger bit, length timer enabled
			0xFF14 => {
				self.frequency = (self.frequency & 0xFF) | (((write & 0x7) as u16) << 8);

				self.length_enabled = ((write >> 6) & 0x1) != 0;

				if self.length_timer == 0 {
					self.length_timer = 64;
				}

				let trigger = (write >> 7) != 0;

				if trigger && self.dac_enabled {
					self.enabled = true;

					self.old_freq = self.frequency;
					self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };

					self.sweep_enabled = self.sweep_period > 0 || self.sweep_amount > 0;

					if self.sweep_amount > 0 {
						self.calc_frequency();
					}

					// reset envelope
					self.envelope_timer = self.envelope_period;
					self.current_volume = self.initial_volume;
				}

			},

			_ => unreachable!()
		}
	}

}

#[derive(Default)]
pub struct SquareChannel2 {

	pub enabled: bool,
	dac_enabled: bool,

	// index into WAVE_DUTY
	duty_pattern: usize,
	// index into WAVE_DUTY[self.wave_pattern]
	pub wave_position: usize,

	// decremented every T-state, increments wave_positoin when it reaches 0
	frequency_timer: u16,
	frequency: u16,

	initial_volume: u8,
	current_volume: u8,

	inc_volume: bool,
	envelope_period: u8,
	envelope_timer: u8,

	length_timer: u16,
	length_enabled: bool,
}

impl SquareChannel2 {

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
			0xFF18 => 0xFF,
			// NR24: upper 3 bits of frequency (write only), trigger bit (write only), length timer enabled
			0xFF19 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

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
				self.length_timer = 64 - (write & 0b0011_1111) as u16;
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