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

fn walk(handle: &Handle, words: Arc<Mutex<HashMap<String, bool>>>, mut in_list: bool, mut pangram: bool) {
    let node = handle;

    match node.data {
        NodeData::Text { ref contents } => {
            if in_list {
                let content = contents.borrow();
                if content.trim() != "" {
                    let mut word_map = words.lock().unwrap();
                    word_map.insert(content.trim().to_string(), pangram);
                    println!("{}: {}", pangram, content.trim());
                }
            }
        },

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            assert!(name.ns == ns!(html));
            // if in_list {
            //     print!("<{}", name.local);
            // }
            if in_list && *name.local == *"strong" {
                pangram = true;
            }
            for attr in attrs.borrow().iter() {
                assert!(attr.name.ns == ns!());
                if *attr.name.local == *"id" && attr.value == Tendril::from("main-answer-list") {
                    in_list = true;
                }
                //print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            //println!(">");
        },

        NodeData::ProcessingInstruction { .. } => unreachable!(),
        _ => {},
    }

    for child in node.children.borrow().iter() {
        walk(child, words.clone(), in_list, pangram);
    }
}

fn fetch_words_from_web(url: &str, words: Arc<Mutex<HashMap<String, bool>>>) -> Result<(), curl::Error> {
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

fn get_letters(words_p: Arc<Mutex<HashMap<String, bool>>>, letters: &mut [char; 7]) {
    let mut words = words_p.lock().unwrap();

    for (k, v) in words {
        println!("{}, {}",k, v);
    }
}

fn main() {
    println!("test");

    let args: Vec<String> = std::env::args().collect();

    let mut letters: [char; 7] = ['\0'; 7];
    let words: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    //let words = Box::leak(Box::new(words));

    if args.len() != 2 {
        eprintln!("usage: cargo run <domain>");
        std::process::exit(1);
    }
    fetch_words_from_web(&args[1], words.clone());
    get_letters(words.clone(), &mut letters);
}
