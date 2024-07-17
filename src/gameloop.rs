use std::sync::{Arc,Mutex};
use std::time::Duration;
use std::collections::BTreeMap;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

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
                        if letters[li] == c {
                            word.push(c);
                            break;
                        }
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
                        }
                    },
                    k => {
                        println!("kd: {:?}", k)
                    },
                };
            },
            _x => {
                //println!("?: {:?}", x);
            },
        }
    }
    return true;
}

fn render(can: &mut Canvas<Window>, letters: &mut [char; 7], word: &mut String, found: &mut BTreeMap<&str, bool>) {
    can.set_draw_color(Color::RGB(0, 0, 0));
    can.clear();

    //let canvsize: (u32, u32) = can.output_size().expect("Could not get canvas size.");

    can.set_draw_color(Color::RGB(0x44, 0x44, 0x44));
    can.fill_rect(Rect::new(60, 95, 90, 90)).unwrap();
    can.fill_rect(Rect::new(160, 95, 90, 90)).unwrap();
    can.fill_rect(Rect::new(10, 195, 90, 90)).unwrap();
    can.fill_rect(Rect::new(210, 195, 90, 90)).unwrap();
    can.fill_rect(Rect::new(60, 295, 90, 90)).unwrap();
    can.fill_rect(Rect::new(160, 295, 90, 90)).unwrap();

    can.fill_rect(Rect::new(310, 10, 320, 460)).unwrap();

    can.set_draw_color(Color::RGB(0x66, 0x66, 0));
    can.fill_rect(Rect::new(110, 195, 90, 90)).unwrap();

    can.present();
}

pub fn gameloop(words_p: Arc<Mutex<Vec<String>>>, letters: &mut [char; 7]) {
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

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut word = String::new();

    let mut found: BTreeMap<&str, bool> = BTreeMap::new();

    let mut words = words_p.lock().unwrap();
    for w in words.iter() {
        found.insert(&w, false);
    }

    loop {
        if !control(&mut event_pump, letters, &mut word, &mut found) {
            break;
        }

        render(&mut canvas, letters, &mut word, &mut found);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMERATE));
    }
}
