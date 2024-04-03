mod processor;
use nannou::prelude::*;
use processor::*;
use std::{
    env,
    fs::File,
    io::{BufReader, Read},
};

const PIXEL_SIZE: f32 = 20.0;

struct Model {
    processor: Processor,
}

fn model(_app: &App) -> Model {
    let args: Vec<String> = env::args().collect();
    let path = format!("rom/{}", args.get(1).expect("No arguments give for ROM."));
    let f = File::open(path).expect("File not found");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    Model {
        processor: Processor::new(&buffer).unwrap(),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.processor.dec_delay_reg();
    for _ in 0..10 {
        if let Err(err) = model.processor.execute_next() {
            println!("{err}")
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();
    let display_buffer = model.processor.get_display_buffer();
    draw.background().color(BLACK);
    for (x, collumn) in display_buffer.iter().enumerate() {
        for (y, pixel) in collumn.iter().enumerate() {
            if pixel {
                let square = Rect::from_w_h(PIXEL_SIZE, PIXEL_SIZE)
                    .top_left_of(win)
                    .shift_x(x as f32 * PIXEL_SIZE)
                    .shift_y(-(y as f32) * PIXEL_SIZE);
                draw.rect().xy(square.xy()).wh(square.wh()).color(GREEN);
            }
        }
    }
    draw.to_frame(app, &frame).unwrap()
}

fn event(_app: &App, model: &mut Model, event: Event) {
    if let Event::WindowEvent {
        id: _,
        simple: Some(event),
    } = event
    {
        match event {
            WindowEvent::KeyPressed(key) => {
                if let Some(key) = match_key(key) {
                    model.processor.set_key(key, true)
                }
            }
            WindowEvent::KeyReleased(key) => {
                if let Some(key) = match_key(key) {
                    model.processor.set_key(key, false)
                }
            }
            _ => {}
        }
    }
}

fn match_key(key: Key) -> Option<usize> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}

fn main() {
    nannou::app(model)
        .size(
            DISPLAY_WIDTH as u32 * PIXEL_SIZE as u32,
            DISPLAY_HEIGHT as u32 * PIXEL_SIZE as u32,
        )
        .simple_window(view)
        .event(event)
        .update(update)
        .run();
}
