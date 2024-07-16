use std::collections::HashMap;
use curl::easy::Easy;
// use std::io::{stdout, Write};
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

    for c in w_iter.next().unwrap().chars() {
        let mut has_letter = false;
        for i in 0..l_part {
            if letters[i] == c {
                has_letter = true;
                break;
            }
        }
        if !has_letter {
            letters[l_part] = c;
            l_part += 1;
        }
    }

    println!("{:?}", letters);

    for w in w_iter {
        let mut i = 0;
        while i < l_part {
            let mut has_letter = false;
            for c in w.chars() {
                if letters[i] == c {
                    has_letter = true;
                    break;
                }
            }
            if has_letter {
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
    fetch_words_from_web(&args[1], words.clone());

    match get_letters(words.clone(), &mut letters) {
        0 => {},
        x => {
            println!("could not determine center letter");
            std::process::exit(1);
            // todo we could prompt user to ask them to select one though idk if this ever occurs
        },
    };
}
