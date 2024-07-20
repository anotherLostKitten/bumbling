//use std::sync::{Arc,Mutex};
use std::time::Duration;
use std::collections::BTreeMap;
use std::path::Path;
use std::string::ToString;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::render::TextureCreator;

const FRAMERATE: u32 = 128;

pub fn control(pump: &mut EventPump, letters: &mut [char; 7], word: &mut String, found: &mut BTreeMap<&str, bool>) -> bool {
    for event in pump.poll_iter() {
        match event {
            Event::Quit {..} => {
                return false;
            },
            Event::TextInput {text, ..} => {
                //println!("te: {:?}", text);
                for c in text.chars() {
                    for li in 0..7 {
                        if letters[li] == c.to_ascii_lowercase() {
                            word.push(c.to_ascii_lowercase());
                            break;
                        }
                        //println!("{}", word);
                    }
                }
            },
            Event::KeyDown {keycode: Some(key), repeat: false, .. } => {
                match key {
                    Keycode::BACKSPACE => {
                        word.pop();
                    },
                    Keycode::RETURN => {
                        //println!("WORD: {}", word);
                        if let Some(vv) = found.get_mut(&word as &str) {
                            //println!("found");
                            *vv = true;
                            word.clear();
                        }
                    },
                    Keycode::ESCAPE => {
                        word.clear();
                    },
                    _k => {
                        //println!("kd: {:?}", _k);
                    },
                };
            },
            _x => {
                //println!("?: {:?}", _x);
            },
        }
    }
    return true;
}

fn render_text_rect(can: &mut Canvas<Window>, tc: &TextureCreator<WindowContext>, font: &mut Font, txt: impl ToString + std::fmt::Display, mut rect: Rect) {
    let surf = font.render(&txt.to_string()).blended(Color::RGBA(0xff, 0xff, 0xff, 0xff)).unwrap();
    let wmul = rect.width() as f32 / surf.width() as f32;
    let hmul = rect.height() as f32 / surf.height() as f32;

    if wmul > hmul {
        let rw = (surf.width() as f32 * hmul) as u32;
        let ro = ((rect.width() - rw) / 2) as i32;
        rect.set_width(rw);
        rect.set_x(rect.x() + ro);
    } else {
        let rh = (surf.height() as f32 * wmul) as u32;
        let ro = ((rect.height() - rh) / 2) as i32;
        rect.set_height(rh);
        rect.set_y(rect.y() + ro);
    }

    let texture = tc.create_texture_from_surface(&surf).unwrap();
    can.copy(&texture, None, Some(rect)).unwrap();
}

fn letrec(i: usize) -> Rect {
    match i {
        0 => Rect::new(110, 195, 90, 90),
        1 => Rect::new(60, 95, 90, 90),
        2 => Rect::new(160, 95, 90, 90),
        3 => Rect::new(10, 195, 90, 90),
        4 => Rect::new(210, 195, 90, 90),
        5 => Rect::new(60, 295, 90, 90),
        6 => Rect::new(160, 295, 90, 90),
        _ => unreachable!(),
    }
}

fn render(can: &mut Canvas<Window>, tc: &TextureCreator<WindowContext>, font: &mut Font, letters: &mut [char; 7], word: &mut String, found: &mut BTreeMap<&str, bool>) {
    can.set_draw_color(Color::RGB(0, 0, 0));
    can.clear();

    //let canvsize: (u32, u32) = can.output_size().expect("Could not get canvas size.");

    can.set_draw_color(Color::RGB(0x44, 0x44, 0x44));
    for i in 1..7 {
        can.fill_rect(letrec(i)).unwrap();
    }

    can.fill_rect(Rect::new(310, 10, 320, 460)).unwrap();

    can.set_draw_color(Color::RGB(0x66, 0x66, 0));
    can.fill_rect(letrec(0)).unwrap();

    for i in 0..7 {
        render_text_rect(can, tc, font,
                         letters[i].to_ascii_uppercase(), letrec(i));
    }

    if word.len() > 0 {
        render_text_rect(can, tc, font, word, Rect::new(10, 10, 290, 75));
    }

    let mut h = 0;
    let mut w = 0;

    let mut f = 0;
    for (ans, isf) in found.iter() {
        if *isf {

            render_text_rect(can, tc, font, ans,
                             Rect::new(310 + w * 80, 10 + h * 20, 80, 20));

            h = (h + 1) % 22;
            if h == 0 {
                w += 1;
            }
            f += 1;
        }
    }

    render_text_rect(can, tc, font, format!("{} / {} found", f, found.size()),

    can.present();
}

pub fn gameloop(found: &mut BTreeMap<&str, bool>, letters: &mut [char; 7]) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();

    let ttf_context = sdl2::ttf::init().unwrap();
    let mut font = ttf_context.load_font(Path::new("Inconsolata.ttf"), 128).unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let window = video_subsystem.window("BumBling", 640, 480)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().accelerated().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut word = String::new();

    loop {
        if !control(&mut event_pump, letters, &mut word, found) {
            break;
        }

        render(&mut canvas, &texture_creator, &mut font, letters, &mut word, found);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMERATE));
    }

    println!("saving...");
}
