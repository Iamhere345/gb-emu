#[derive(Default)]
pub struct NoiseChannel {

	pub enabled: bool,
	dac_enabled: bool,

	// linear feedback shift register
	lfsr: u16,

	// rng register
	nr43: u8,

	// decremented every T-state, increments wave_position when it reaches 0
	frequency_timer: u16,

	initial_volume: u8,
	current_volume: u8,

	inc_volume: bool,
	envelope_period: u8,
	envelope_timer: u8,

	length_timer: u16,
	length_enabled: bool,
}

impl NoiseChannel {

	pub fn tick(&mut self) {

		if self.frequency_timer == 0 {
			let divisor_code = (self.nr43 & 0x07) as u16;

            self.frequency_timer = (if divisor_code == 0 { 8 } else { divisor_code << 4 }) << ((self.nr43 >> 4) as u32);

            let xor_result = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);

            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

            if ((self.nr43 >> 3) & 0b01) != 0 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }

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

			let dac_input = (!self.lfsr & 0x1) as f32 * self.current_volume as f32;

			(dac_input / 7.5) - 1.0
		} else {
			0.0
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			// NR40: does not exist
			0xFF1F => 0xFF,
			// NR41: initial length timer (write-only)
			0xFF20 => 0xFF,
			// NR42: initial volume & envelope
			0xFF21 => (self.initial_volume << 4) | ((self.inc_volume as u8) << 3) | self.envelope_period,
			// NR43: frequency & randomness
			0xFF22 => self.nr43,
			// NR44: control register
			0xFF23 => (self.length_enabled as u8) << 5,

			_ => unreachable!()
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			//NR40: does not exist
			0xFF1F => {},
			// NR41: duty patten & initial length timer
			0xFF20 => self.length_timer = (64 - (write & 0b0011_1111)) as u16,
			// NR42: initial volume & envelope
			0xFF21 => {
				self.initial_volume = write >> 4;
				self.inc_volume = (write & 0x8) != 0;
				self.envelope_period = write & 0x7;

				self.dac_enabled = (write & 0b1111_1000) != 0;
			},
			// NR43: frequency & randomness
			0xFF22 => self.nr43 = write,
			// NR44: upper 3 bits of frequency, trigger bit, length timer enabled
			0xFF23 => {
				self.length_enabled = ((write >> 6) & 0x1) != 0;

				if self.length_timer == 0 {
					self.length_timer = 64;
				}

				let trigger = (write >> 7) != 0;

				if trigger && self.dac_enabled {
                    self.enabled = true;
                }

				if trigger {
					self.lfsr = 0x7FFF;

					// reset envelope
					self.envelope_timer = self.envelope_period;
					self.current_volume = self.initial_volume;
				}

			},

			_ => unreachable!()
		}
	}

}