# gb-emu

Yet another Gameboy (DMG) emulator, written in rust. Has a UI, debugger, and is accurate enough to run most games.

---

| [dmg-acid2](https://github.com/mattcurrie/dmg-acid2)                                     | Legend of Zelda - Link's Awakening                                                        | Pokemon Red                                                                               | Super Mario Land                                                                          | Metroid II: Return of Samus                                                               | Kid Icarus: Of Myths and Monsters                                                         | [Blargg's Test Roms - cpu_instrs.gb](https://gbdev.gg8.se/files/roms/blargg-gb-tests/)    |
| ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
|![image](https://github.com/user-attachments/assets/1e8e5719-a8f2-4f81-ba03-9cd741eb7849) | ![image](https://github.com/user-attachments/assets/139564ae-9bce-413b-b8c1-2fd3080ad025) | ![image](https://github.com/user-attachments/assets/cf6b783c-bae9-46b5-8e95-301ba617d7c0) | ![image](https://github.com/user-attachments/assets/c3392eef-28bb-4a3b-a0d7-d94121333682) | ![image](https://github.com/user-attachments/assets/4ff35502-469e-48a7-9982-d3779123d5ed) | ![image](https://github.com/user-attachments/assets/7107dfba-24c8-49d0-916f-12100361d2c6) | ![image](https://github.com/user-attachments/assets/f19ccbc9-8f7f-4d4e-a93f-85a667ccca9e) |

## Features

 - [x] DMG support
 - [x] MBC1
 - [x] MBC2
 - [X] MBC3
 - [X] MBC5
 - [X] Sound
 - [x] Controller support via `gilrs`
 - [x] Scanline-based renderer (no pixel FIFO)
 - [X] `Egui` ui frontend
 - [x] built-in debugger
 - [x] Game saves on battery-backed cartridges
 - [ ] CGB (Gameboy Colour) support
 - [ ] MBC3 RTC
 - [ ] MBC5 controller rumble
 - [ ] Cycle-accurate CPU
 - [ ] Pixel FIFO
 - [ ] Libretro core

## Usage

### Building
Compile from source using `cargo build --release` or download a pre-built binary from the [releases](https://github.com/Iamhere345/gb-emu/releases) tab.

### Controls

#### Keyboard:
 - Z: A
 - X: B
 - Arrow keys: Dpad
 - Enter: Start
 - Backspace: Select

#### Controller:
 - South / X / A: A
 - East / O / B: B
 - Left stick / Dpad: Dpad
 - Start: Start
 - Select / Share: Select

### Bootroms
To use your own bootrom, name your bootrom file `bootrom.gb` and place it in the `roms/` folder next to the executable. In the emulator, check the `Enable bootrom` checkbox.

## Resources

Here are some of the resources I used to make this emulator:
 - [The Pan Docs](https://gbdev.io/pandocs/)
 - [GB CPU Opcode Table](https://gbdev.io/gb-opcodes/optables/)
 - [RGBDS Opcode Reference](https://rgbds.gbdev.io/docs/v0.8.0/gbz80.7)
 - [The Ultimate Gameboy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI)
 - [Blargg's Test Roms](https://gbdev.gg8.se/files/roms/blargg-gb-tests/)
 - [The Mooneye Test Suite](https://gekkio.fi/files/mooneye-gb/latest/)
 - [dmg-acid2 PPU Test](https://github.com/mattcurrie/dmg-acid2)
 - [The EmuDev subreddit](https://www.reddit.com/r/EmuDev/)
 - [The EmuDev discord](https://discord.gg/dkmJAes)
 - [GBdev Community](https://gbdev.io/)
 - [The Gameboy Development Guide](https://hacktix.github.io/GBEDG/)
 - [Nightshade's Blog](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html) and [his emulator](https://github.com/NightShade256/Argentum) (used to implement most of the APU) 
