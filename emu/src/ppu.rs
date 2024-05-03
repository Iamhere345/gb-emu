
enum RenderingMode {
    HBlank,     // Mode 0
    VBlank,     // Mode 1
    OAMscan,    // Mode 2
    Draw,       // Mode 3   
}

struct PPU {
    rendering_mode: RenderingMode,
    vram: [u8; 0x2000], // 8k
    oam: [u8; 160]
}

impl PPU {

    pub fn new() -> Self {
        Self {
            rendering_mode: RenderingMode::VBlank,
            vram: [0; 0x2000],
            oam: [0; 160]
        }
    }

    pub fn read(addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram[addr - 0x8000],
            0xFE00..=0xFE9F => self.oam[addr - 0xFE00]
        }
    }

    pub fn write(addr: u16, write: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[addr - 0x8000] = write,
            0xFE00..=0xFE9F => self.oam[addr - 0xFE00] = write
        }
    }

}
