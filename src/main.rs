extern crate clap;
extern crate failure;
extern crate git2;
extern crate pulldown_cmark;

use clap::{App, Arg};
use failure::{Error, bail};

use std::{
    fs::File,
    io::{self, Read},
};

use two_trucs::rewrite;

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
                .help("Set the title for the new day"),
        )
        .arg(
            Arg::with_name("input")
                .index(1)
                .help("The TODO file to process"),
        )
        .arg(
            Arg::with_name("inplace")
                .long("inplace")
                .help("Write to the input file instead of printing to stdout"),
        )
        .arg(
            Arg::with_name("git")
                .long("git")
                .help("Automatically commit the file to a git repo (implies `--inplace`)"),
        )
        .get_matches();

    if matches.is_present("inplace") || matches.is_present("git") {
        match matches.value_of("input") {
            Some("-") | None =>
                bail!("Cannot use `--inplace` or `--git` without a filename"),
            _ => (),
        }
    }

    let mut input = String::new();
    match matches.value_of("input") {
        Some("-") | None => {
            io::stdin().read_to_string(&mut input)?;
        }
        Some(path) => {
            let mut f = File::open(path)?;
            f.read_to_string(&mut input)?;
        }
    }

    let opt_title = if matches.is_present("next") {
        matches.value_of("title")
    } else {
        None
    };

    if matches.is_present("inplace") || matches.is_present("git") {
        let file_path = match matches.value_of("input") {
            Some("-") | None => unreachable!(),
            Some(path) => path,
        };
        let mut output_file = File::create(file_path)?;
        rewrite::rewrite(opt_title, &input, &mut output_file)?;

        if matches.is_present("git") {
            let repo = git2::Repository::discover(file_path)?;
            let mut index = repo.index()?;
            let canonical_path = std::path::Path::new(file_path).canonicalize()?;
            let relative_path = canonical_path.strip_prefix(repo.workdir().unwrap())?;
            index.add_path(relative_path)?;
            let tree = repo.find_tree(index.write_tree()?)?;
            let sig = repo.signature()?;
            let head = repo.head()?.peel_to_commit()?;
            let msg = format!("two-trucs: Updating todo file `{}`", relative_path);
            repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&head])?;
        }
    } else {
        rewrite::rewrite(opt_title, &input, &mut io::stdout())?;
    }

    Ok(())
}
