use std::collections::HashMap;
use curl::easy::Easy;
 use std::io::Write;
use std::sync::{Arc,Mutex};

#[macro_use]
extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::{TendrilSink, Tendril};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

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

fn get_letters(words_p: Arc<Mutex<Vec<String>>>, letters: &mut [char; 7]) -> usize {
    let words = words_p.lock().unwrap();
    let mut w_iter = words.iter();

    let mut l_part = 0;

    let mut lset: u32 = 0;

    for c in w_iter.next().unwrap().chars() {
        if lset & 1 << (c as u32 & 31) == 0 {
            lset |= 1 << (c as u32 & 31);
            letters[l_part] = c;
            l_part += 1;
        }
    }

    println!("{:?}", letters);

    for w in w_iter {
        let mut lset: u32 = 0;
        for c in w.chars() {
            lset |= 1 << (c as u32 & 31);
        }

        let mut i = 0;
        while i < l_part {
            if lset & 1 << (letters[i] as u32 & 31) != 0 {
                i += 1;
            } else {
                println!("word {} missing {}", w, letters[i]);
                let tmp = letters[i];
                l_part -= 1;
                letters[i] = letters[l_part];
                letters[l_part] = tmp;
            }
        }
    }

    println!("{}", l_part);
    println!("{:?}", letters);

    return l_part;
}

fn main() {
    println!("test");

    let args: Vec<String> = std::env::args().collect();

    let mut letters: [char; 7] = ['\0'; 7];
    let words: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    if args.len() != 2 {
        eprintln!("usage: cargo run <domain>");
        std::process::exit(1);
    }

    match fetch_words_from_web(&args[1], words.clone()) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        },
    };

    match get_letters(words.clone(), &mut letters) {
        0 => {
            eprintln!("error: no possible valid center letter");
            std::process::exit(1);
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
                        let tmp = letters[0];
                        letters[0] = letters[i];
                        letters[i] = tmp;
                        break 'get_center_letter;
                    }
                }
            }
        },
    };
}
