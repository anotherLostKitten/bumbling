use curl::easy::Easy;
use std::io::Write;
use std::sync::{Arc,Mutex};
use std::path::Path;

#[macro_use]
extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::{TendrilSink, Tendril};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

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

fn get_letters(words_p: Arc<Mutex<Vec<String>>>, letters: &mut [char; 7]) -> bool {
    let mut words = words_p.lock().unwrap();

    let mut l_part = 0;

    let mut lset_max: u32 = 0;

    'letter_collect: for w in words.iter() {
        for c in w.chars() {
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

    let mut pgram_count = 0;

    for wi in 0..words.len() {
        let w = &words[wi];

        let mut lset: u32 = 0;
        for c in w.chars() {
            lset |= 1 << (c as u32 & 31);
        }

        if lset & !lset_max != 0 {
            eprintln!("error: word set has more than 7 letters");
        }

        let mut i = 0;
        while i < l_part {
            if lset & 1 << (letters[i] as u32 & 31) != 0 {
                i += 1;
            } else {
                // println!("word {} missing {}", w, letters[i]);
                l_part -= 1;
                if i != l_part {
                    letters.swap(i, l_part);
                }
            }
        }

        if lset == lset_max {
            if wi > pgram_count {
                words.swap(wi, pgram_count);
            }
            pgram_count += 1;
        }

    }

    // println!("pangrams: {}", pgram_count);
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

fn usage(n: usize) {
    if n > 0 {
        eprintln!("error: at token #{}", n);
    }
    eprintln!(concat!("usage: cargo run ((", argmar!(), "w|", argmar!(), "s) <url> <path>? | ", argmar!(), "f <path> | ", argmar!(), "g <word>+)+"));
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut file_counter = 0;

    if args.len() < 2 {
        usage(1);
    }
    let mut argi = 1;
    println!("{:?}", args);
    while argi < args.len() {
        let mut letters: [char; 7] = ['\0'; 7];
        let words: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

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

                let mut strloc;
                let path = if argi < args.len() && !args[argi].starts_with(argmar!()) {
                    argi += 1;
                    Path::new(&args[argi - 1])
                } else {
                    loop {
                        strloc = format!("default_file_{}.bumble", file_counter);
                        let path_tst = Path::new(&strloc);
                        if !path_tst.exists() {
                            break path_tst
                        }
                        file_counter += 1;
                    }
                };

                if let Err(e) = fetch_words_from_web(&url, words.clone()) {
                    eprintln!("error: {}", e);
                    continue;
                }

                get_letters(words.clone(), &mut letters);

                if let Err(e) = std::fs::write(path, format!("{}", words.clone().lock().unwrap().join("\n"))) {
                    eprintln!("could not write file: {}", e);
                }

                if v == concat!(argmar!(), "s") {
                    continue;
                }
                gameloop::gameloop(words.clone(), &mut letters);
            },
            concat!(argmar!(), "f") => {
                let path = if argi < args.len() && !args[argi].starts_with(argmar!()) {
                    argi += 1;
                    Path::new(&args[argi - 1])
                } else {
                    usage(argi);
                    unreachable!();
                };

                if let Ok(src) = std::fs::read_to_string(path) {
                    {
                        let mut words = words.lock().unwrap();
                        for w in src.split("\n") {
                            words.push(w.to_string());
                        }
                    }
                    get_letters(words.clone(), &mut letters);

                    gameloop::gameloop(words.clone(), &mut letters);
                } else {
                    eprintln!("could not read file {}", path.display());
                    continue;
                }
            },
            concat!(argmar!(), "g") => {
                eprintln!("workin on it");
            },
            _ => {usage(argi - 1);},
        }
    }

}
