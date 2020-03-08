extern crate clap;
extern crate comrak;
extern crate failure;
extern crate pulldown_cmark;

use clap::{App, Arg};
use comrak::{Arena, ComrakOptions};
use failure::Error;

use std::{
    fs::File,
    io::{self, Read},
};

mod next;
mod parse;
mod render;
mod sort;
mod utils;

fn main() -> Result<(), Error> {
    let matches = App::new("updo")
        .version("0.1.0")
        .author("Trevor Elliott")
        .about("Markdown TODO list maintainer")
        .arg(
            Arg::with_name("next")
                .short("n")
                .long("next")
                .help("Start a new day"),
        )
        .arg(
            Arg::with_name("title")
            .short("t")
            .long("title")
            .takes_value(true)
            .default_value("Today")
            .help("Set the title for the new day"))
        .arg(
            Arg::with_name("input")
                .index(1)
                .help("The TODO file to process"),
        )
        .get_matches();

    let mut buf = String::new();
    let input = matches.value_of("input");
    match input {
        Some("-") | None => {
            io::stdin().read_to_string(&mut buf)?;
        }
        Some(path) => {
            let mut f = File::open(path)?;
            f.read_to_string(&mut buf)?;
        }
    }

    let arena = Arena::new();

    let opts = {
        let mut opts = ComrakOptions::default();
        opts.ext_tasklist = true;
        opts
    };

    let doc = comrak::parse_document(&arena, &buf, &opts);

    sort::sort_tasks(doc);

    if matches.is_present("next") {
        next::start_next_day(&arena, doc, matches.value_of("title").unwrap())?;
    }

    render::render_document(doc, &opts, &mut io::stdout())?;

    Ok(())
}
