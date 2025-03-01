use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::WindowCanvas, Sdl,
    VideoSubsystem,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    thread, time,
};

mod chip_8;
use chip_8::{Chip8, Chip8Builder, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const PIXEL_SIZE: u32 = 10;

fn main() {
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Rust Chip-8;",
            DISPLAY_WIDTH as u32 * PIXEL_SIZE,
            DISPLAY_HEIGHT as u32 * PIXEL_SIZE,
        )
        .position_centered()
        .build();

    let mut canvas: WindowCanvas = window.unwrap().into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let program = load_program_bytes();
    let mut chip_8: Chip8 = Chip8Builder::new()
        .with_program(&program)
        .with_vf_reset_quirk()
        .with_jumping_quirk()
        .build()
        .unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            use Event::*;
            match event {
                Quit { .. }
                | KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(key) = sdl_key_to_ch8_key(key) {
                        chip_8.set_key(key);
                    }
                }
                KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(key) = sdl_key_to_ch8_key(key) {
                        chip_8.unset_key(key);
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 255, 0));

        let display_buffer = chip_8.get_display_buffer();
        let pixel_size = if chip_8.is_high_res() {
            PIXEL_SIZE
        } else {
            PIXEL_SIZE * 2
        } as i32;
        for (x, collumn) in display_buffer.iter().enumerate() {
            for (y, pixel) in collumn.iter().enumerate() {
                if pixel {
                    canvas
                        .fill_rect(Rect::new(
                            x as i32 * pixel_size,
                            y as i32 * pixel_size,
                            pixel_size as u32,
                            pixel_size as u32,
                        ))
                        .unwrap();
                }
            }
        }
        chip_8.dec_delay_reg();
        for _ in 0..1000 {
            let emu_res = chip_8.execute_next();
            if let Err(err) = emu_res {
                println!("{err}");
            }
        }
        canvas.present();
        thread::sleep(time::Duration::from_millis(16));
    }
}

fn load_program_bytes() -> Box<[u8]> {
    let args: Box<[String]> = std::env::args().collect();
    let path = format!("rom/{}", args.get(1).expect("No argument given for ROM."));
    let file = File::open(path).expect("File not found");
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    buffer.into_boxed_slice()
}

fn sdl_key_to_ch8_key(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Kp1 => Some(0x1),
        Keycode::Kp2 => Some(0x2),
        Keycode::Kp3 => Some(0x3),
        Keycode::Kp4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
