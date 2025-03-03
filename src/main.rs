use std::io::Write;
use std::sync::{Arc,Mutex};
use std::path::Path;
use std::collections::BTreeMap;

use curl::easy::Easy;
use chrono::Utc;
use chrono_tz::US::Pacific;
use regex::Regex;

#[macro_use]
extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::{TendrilSink, Tendril};
use rcdom::{Handle, NodeData, RcDom};

//use dev_tools::*;

mod gameloop;

macro_rules! argmar {
    () => {"_"};
}

fn walk(handle: &Handle, words: Arc<Mutex<Vec<String>>>, mut in_list: bool, mut pangram: bool) {
    let node = handle;

    match node.data {
        NodeData::Text { ref contents } => {
            if in_list {
                let content = contents.borrow();
                if content.trim() != "" {
                    let mut word_vec = words.lock().unwrap();
                    if pangram {
                        word_vec.insert(0, content.trim().to_string());
                    } else {
                        word_vec.push(content.trim().to_string());
                    }
                    // println!("{}: {}", pangram, content.trim());
                }
            }
        },

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            assert!(name.ns == ns!(html));
            if in_list && *name.local == *"strong" {
                pangram = true;
            }
            for attr in attrs.borrow().iter() {
                assert!(attr.name.ns == ns!());
                if *attr.name.local == *"id" && attr.value == Tendril::from("main-answer-list") {
                    in_list = true;
                }
            }
        },

        NodeData::ProcessingInstruction { .. } => unreachable!(),
        _ => {},
    }

    for child in node.children.borrow().iter() {
        walk(child, words.clone(), in_list, pangram);
    }
}

fn fetch_words_from_web(url: &str, words: Arc<Mutex<Vec<String>>>) -> Result<(), curl::Error> {
    let mut curler = Easy::new();
    curler.url(url)?;
    curler.write_function(move |data| {

        // stdout().write_all(data).unwrap();
        // return Ok(data.len());

        let tendril = Tendril::try_from_byte_slice(data).unwrap();

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .one(tendril);

        walk(&dom.document, words.clone(), false, false);

        Ok(data.len())
    })?;
    curler.perform()?;

    Ok(())
}

fn get_letters<'a: 'b, 'b>(words: &'a mut Vec<String>, letters: &mut [char; 7], found: &'b mut BTreeMap<&'a str, bool>) -> bool {
    let mut l_part = 0;

    let mut lset_max: u32 = 0;

    'letter_collect: for w in words.iter() {
        if w == "" {
            continue;
        }
        for c in w.chars() {
            if c < 'a' || c > 'z' {
                continue
            }
            if lset_max & 1 << (c as u32 & 31) == 0 {
                lset_max |= 1 << (c as u32 & 31);
                letters[l_part] = c;
                l_part += 1;

                if l_part == 7 {
                    break 'letter_collect;
                }
            }
        }
    }

    if l_part < 7 {
        eprintln!("error: word set has less than 7 letters");
        return false;
    }

    //println!("{:?}", letters);

    let mut _pgram_count = 0;

    for wi in 0..words.len() {
        let w = &words[wi];

        if w == "" {
            continue;
        }

        let mut isfound = false;

        let mut lset: u32 = 0;
        let mut ci = 0;
        for c in w.chars() {
            if c == '+' {
                isfound = true;
                continue;
            }
            lset |= 1 << (c as u32 & 31);
            ci += 1;
        }

        if lset & !lset_max != 0 {
            eprintln!("error: word set has more than 7 letters");
        }

        let mut i = 0;
        while i < l_part {
            if lset & 1 << (letters[i] as u32 & 31) != 0 {
                i += 1;
            } else {
                //println!("word {} missing {}", w, letters[i]);
                l_part -= 1;
                if i != l_part {
                    letters.swap(i, l_part);
                }
            }
        }

        if lset == lset_max {
            _pgram_count += 1;
        }

        found.insert(&words[wi][..ci], isfound);
    }

    // println!("pangrams: {}", _pgram_count);
    // println!("letters: {:?}", letters);

    match l_part {
        0 => {
            eprintln!("error: no possible valid center letter");
            return false;
        },
        1 => {},
        x => {
            'get_center_letter: loop {
                print!("could not determine center letter, please enter [");
                for i in 0..x {
                    print!("{}", letters[i]);
                }
                print!("]:");
                std::io::stdout().flush().unwrap();

                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                //println!("{}", buf);
                let c = match buf.chars().next() {
                    Some(c) => c,
                    None => {continue;}
                };
                for i in 0..x {
                    if c == letters[i] {
                        if i != 0 {
                            letters.swap(0, i);
                        }
                        break 'get_center_letter;
                    }
                }
            }
        },
    };
    return true;
}

fn write_save(found: &BTreeMap<&str, bool>, path: &Path) {
    let mut res = String::new();

    for (w, fnd) in found {
        if *w == "" {
            continue;
        }
        res.push_str(w);
        if *fnd {
            //println!("found {}", w);
            res.push('+');
        }
        res.push('\n');
    }

    if let Err(e) = std::fs::write(path, res) {
        eprintln!("could not write file: {}", e);
    }
}

fn run_game_from_file(path: &Path) {
    let mut letters: [char; 7] = ['\0'; 7];
    let words: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut found: BTreeMap<&str, bool> = BTreeMap::new();

    if let Ok(src) = std::fs::read_to_string(path) {
        let mut words = words.lock().unwrap();
        for w in src.split("\n") {
            words.push(w.to_string());
        }

        get_letters(&mut words, &mut letters, &mut found);

        gameloop::gameloop(&mut found, &mut letters);

        write_save(&found, path);
    } else {
        eprintln!("could not read file {}", path.display());
    }
}

fn run_game_from_web(url: &str, path: &Path, save_only: bool) {
    let mut letters: [char; 7] = ['\0'; 7];
    let words: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut found: BTreeMap<&str, bool> = BTreeMap::new();

    if let Err(e) = fetch_words_from_web(&url, words.clone()) {
        eprintln!("error: {}", e);
        return;
    }

    let mut words = words.lock().unwrap();

    get_letters(&mut words, &mut letters, &mut found);

    if !save_only {
        gameloop::gameloop(&mut found, &mut letters);
    }

    write_save(&found, path);

}

fn usage(n: usize) {
    if n > 0 {
        eprintln!("error: at token #{}", n);
    }
    eprintln!(concat!("usage: ./bumbling ((", argmar!(), "w|", argmar!(), "s) <url> <path>? | ", argmar!(), "f <path>)*"));
    std::process::exit(1);
}

fn main() {
    let today = format!("{}", Utc::now().with_timezone(&Pacific).format("%Y%m%d"));
    //println!("{}", today);
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        let strloc = format!("{}.bumble", today);
        let path = Path::new(&strloc);

        if path.exists() {
            run_game_from_file(path);
        } else {
            run_game_from_web("https://nytbee.com", path, false);
        }

        return;
    }

    let mut argi = 1;
    println!("{:?}", args);
    while argi < args.len() {
        argi += 1;
        match args[argi - 1].as_str() {
            v @ (concat!(argmar!(), "w") | concat!(argmar!(), "s")) => {
                if argi >= args.len() {
                    usage(argi);
                }
                let url = &args[argi];
                if url.starts_with(argmar!()) {
                    usage(argi);
                }
                argi += 1;

                let strloc;
                let path = if argi < args.len() && !args[argi].starts_with(argmar!()) {
                    argi += 1;
                    Path::new(&args[argi - 1])
                } else {
                    let re = Regex::new(r"Bee_([0-9]{8}).html").unwrap();
                    if let Some(caps) = re.captures(url) {
                        strloc = format!("{}.bumble", &caps[1]);
                        Path::new(&strloc)
                    } else {
                        strloc = format!("{}.bumble", today);
                        Path::new(&strloc)
                    }
                };

                run_game_from_web(&url, &path, v.as_bytes()[1] == 's' as u8);
            },
            concat!(argmar!(), "f") => {
                let path = if argi < args.len() && !args[argi].starts_with(argmar!()) {
                    argi += 1;
                    Path::new(&args[argi - 1])
                } else {
                    usage(argi);
                    unreachable!();
                };

                run_game_from_file(path);
            },
            _ => {usage(argi - 1);},
        }
    }

}
