use sdl2::pixels::Color; 
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::fs;
use std::time::{Duration, Instant};

use emu::Gameboy;
use emu::ppu::GBColour;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

const UPSCALE_FACTOR: u32 = 4;

const CYCLES_PER_FRAME: usize = (4194304.0 / 60.0) as usize;

fn main() {

    let sdl_ctx = sdl2::init().unwrap();
    let video_sys = sdl_ctx.video().unwrap();

    let window = video_sys.window("GB Emulator", SCREEN_WIDTH * UPSCALE_FACTOR, SCREEN_HEIGHT * UPSCALE_FACTOR)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    //canvas.set_draw_color(Color::RGB(155, 188, 15));
    //canvas.clear();
    //canvas.present();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut emu = Gameboy::new();
    //emu.init(include_bytes!("../../dmg-acid2.gb"));
    //emu.init(include_bytes!("../../tests/cpu_instrs/individual/04-op r,imm.gb"));
    //emu.init(include_bytes!("../../tests/cpu_instrs/cpu_instrs.gb"));

    emu.init(&fs::read("roms/tetris.gb").unwrap());

    let mut last_update = Instant::now();

    'running: loop {

        for event in event_pump.poll_iter() {

            match event {
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },

                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::BtnA);
                },
                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::BtnB);
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::BtnStart);
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::BtnSelect);
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::DPadUp);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::DPadDown);
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::DPadLeft);
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    emu.bus.borrow_mut().joypad.btn_down(emu::joypad::GBInput::DPadRight);
                },
                
                Event::KeyUp { keycode: Some(Keycode::Z), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::BtnA);
                },
                Event::KeyUp { keycode: Some(Keycode::X), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::BtnB);
                },
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::BtnStart);
                },
                Event::KeyUp { keycode: Some(Keycode::Backspace), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::BtnSelect);
                },
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::DPadUp);
                },
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::DPadDown);
                },
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::DPadLeft);
                },
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                    emu.bus.borrow_mut().joypad.btn_up(emu::joypad::GBInput::DPadRight);
                },

                _ => {}
            }

        }

        if last_update.elapsed() >= Duration::from_secs_f64(1.0 / 60.0) {

			let mut frames = last_update.elapsed().as_secs_f64();
           
            while frames >= 1.0 / 60.0 {

				let mut cycles: u64 = 0;

				while cycles < CYCLES_PER_FRAME as u64 {
					cycles += emu.tick();
				}

				frames -= CYCLES_PER_FRAME as f64;
			}

			last_update = Instant::now();
		}

        for (i, pixel) in emu.bus.borrow().ppu.pixel_buf.iter().enumerate() {

            let x = i % SCREEN_WIDTH as usize;
            let y = i / SCREEN_WIDTH as usize;

            match pixel {
                GBColour::Black => canvas.set_draw_color(Color::RGB(0, 0, 0)),
                GBColour::DarkGrey => canvas.set_draw_color(Color::RGB(64, 64, 64)),
                GBColour::LightGrey => canvas.set_draw_color(Color::RGB(128, 128, 128)),
                GBColour::White => canvas.set_draw_color(Color::RGB(255, 255, 255)),
            }

            canvas.fill_rect(Rect::new((x as u32 * UPSCALE_FACTOR) as i32, (y as u32 * UPSCALE_FACTOR) as i32, UPSCALE_FACTOR, UPSCALE_FACTOR)).expect("Unable to draw pixel");

        }

        canvas.present();

    }

}
