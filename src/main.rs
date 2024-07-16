use std::collections::HashMap;
use curl::easy::Easy;
use scraper::{Html, Selector};
use std::io::{stdout, Write};

fn fetch_words_from_web(url: &str, words: &mut HashMap<&str, bool>) -> Result<(), curl::Error> {
    let mut curler = Easy::new();
    curler.url(url)?;
    curler.write_function(|data| {

        // stdout().write_all(data).unwrap();
        // return Ok(data.len());

        let str_data = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => {return Ok(0)},
        };
        //println!("{}", str_data);
        let doc = Html::parse_fragment(str_data);
        let sel = Selector::parse("#main-answer-list").unwrap();

        println!("{}", doc.html());

        let val = doc.select(&sel).next().unwrap();
        println!("{}", val.html());

        Ok(data.len())
    })?;
    curler.perform()?;

    Ok(())
}

fn main() {
    println!("test");

    let args: Vec<String> = std::env::args().collect();

    let mut letters: [char; 7];
    let mut words: HashMap<&str, bool> = HashMap::new();

    if args.len() != 2 {
        eprintln!("usage: cargo run <domain>");
        std::process::exit(1);
    }
    fetch_words_from_web(&args[1], &mut words);
}
