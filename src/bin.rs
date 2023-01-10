use std::io::{stdin, Read};
use std::{env, fs, str};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut scriptifier = mimicui::HTMLScriptifier::new();
    let mut buffer = Vec::new();

    if args.len() < 2 {
        println!("Usage: mimicui < - | FILE >");
        std::process::exit(-1);
    }

    let source = args.get(1).unwrap();

    if source == "-" {
        stdin()
            .read_to_end(&mut buffer)
            .expect("Nothing to process!");
    } else {
        buffer = Vec::from(fs::read_to_string("fragment.html").unwrap());
    }

    print!(
        "{}",
        scriptifier.scriptify_html(str::from_utf8(&buffer).unwrap())
    );

    std::process::exit(0);
}
