#[derive(Default)]
pub struct WaveChannel {
	pub enabled: bool,
	dac_enabled: bool,

	wave_ram: [u8; 16],
	wave_position: usize,

	// encoded version of volume shift
	output_level: u8,
	// the amount by which the sample is shifted right
	volume_shift: u8,

	// decremented every T-state, increments wave_positoin when it reaches 0
	frequency_timer: u16,
	frequency: u16,

	length_timer: u16,
	length_enabled: bool,
}

impl WaveChannel {

	pub fn tick(&mut self) {

		if self.frequency_timer == 0 {

			self.frequency_timer = (2048 - self.frequency) * 2;

			// wraps around after 31
			self.wave_position = (self.wave_position + 1) % 32;

		}

		self.frequency_timer -= 1;

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

			let dac_input = self.wave_ram[self.wave_position / 2]
				>> (if self.wave_position % 2 == 0 { 4 } else { 0 })
				& 0xF;

			((dac_input >> self.volume_shift) as f32 / 7.5) - 1.0
		} else {
			0.0
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		match addr {
			// NR30: DAC enable
			0xFF1A => ((self.dac_enabled as u8) << 7) | 0x7F,
			// NR31: Length timer (write-only)
			0xFF1B => 0xFF,
			// NR33: output level
			0xFF1C => (self.output_level << 5) | 0x9F,
			// NR33: period low (write-only)
			0xFF1D => 0xFF,
			// NR24: upper 3 bits of frequency (write only), trigger bit (write only), length timer enabled
			0xFF1E => ((self.length_enabled as u8) << 6) | 0b1011_1111,
			// Wave RAM
			0xFF30 ..= 0xFF3F => self.wave_ram[addr as usize - 0xFF30],

			_ => unreachable!()
		}
	}

	pub fn write(&mut self, addr: u16, write: u8) {
		match addr {
			// NR30: DAC enable
			0xFF1A => {
				self.dac_enabled = (write >> 7) & 0b1 != 0;

				if !self.dac_enabled {
					self.enabled = false;
				}
			},
			// NR31: Length timer
			0xFF1B => self.length_timer = 256 - write as u16,
			// NR33: output level
			0xFF1C => {
				self.output_level = (write >> 5) & 0b11;

				self.volume_shift = match self.output_level {
                    0b00 => 4,
                    0b01 => 0,
                    0b10 => 1,
                    0b11 => 2,

                    _ => unreachable!(),
                };
			},
			// NR33: lower 8 bits of the frequency (write only)
			0xFF1D => {
				self.frequency = (self.frequency & 0x700) | write as u16;
			},
			// NR34: upper 3 bits of frequency, trigger bit, length timer enabled
			0xFF1E => {
				self.frequency = (self.frequency & 0xFF) | (((write & 0x7) as u16) << 8);

				self.length_enabled = ((write >> 6) & 0x1) != 0;

				if self.length_timer == 0 {
					self.length_timer = 64;
				}

				let trigger = (write >> 7) != 0;

				if trigger && self.dac_enabled {
					self.enabled = true;
				}

			},
			// Wave RAM
			0xFF30 ..= 0xFF3F => self.wave_ram[addr as usize - 0xFF30] = write,

			_ => unreachable!()
		}
	}

}