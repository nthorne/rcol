use clap::Clap;
use std::{io, io::prelude::*};
use std::collections::HashMap;
use regex::Regex;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Niklas Thorne <notrupertthorne AT gmail>")]
struct Opts {
    #[clap(name="INPUT", default_value="-", about="Input file to colorize. Defaults to stdin.")]
    input: String,
    #[clap(short, long, default_value="[ \t]+", about="The column delimiter to use.")]
    delimiter: String,
    #[clap(short, long, default_value="0", about="Which column to utilize for colorization")]
    column: u32
}

type Color = u32;
type ColorMap = HashMap::<String, Color>;

fn parse_line(line: &str, delimiter: &Regex, column: u32, map: &mut ColorMap) -> Color {
    let results = delimiter.split(line).collect::<Vec<&str>>();
    println!("after split: {:?}", results);
    0
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);

    let mut color_map = ColorMap::new();

    let rex = Regex::new(opts.delimiter.as_str()).unwrap();

    if opts.input == "-" {
        for line in io::stdin().lock().lines() {
            if let Ok(l) = line {
                let _color = parse_line(&l, &rex, opts.column, &mut color_map);
            } else {
                println!("Failed to read line");
            }
        }
    }
}
