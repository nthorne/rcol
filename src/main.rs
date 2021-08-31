use structopt::StructOpt;
use std::{io, io::BufReader, io::prelude::*};
use std::collections::HashMap;
use regex::Regex;
use ansi_term::Colour::Fixed;
use std::fs::File;

#[derive(StructOpt, Debug)]
#[structopt(
    author = "Niklas Thorne <notrupertthorne AT gmail>",
    about="Colorize lines from a file, or stdin, by grouping lines according to a given delimiter and column.")]
struct Opts {
    /// Input file to colorize. Defaults to stdin.
    #[structopt(name="INPUT", default_value="-")]
    input: String,

    /// The column delimiter to use.
    #[structopt(short, long, default_value="[ \t]+")]
    delimiter: String,

    /// Which column to utilize for colorization.
    #[structopt(short, long, default_value="0")]
    column: u32,

    /// List of colors to not use when coloring.
    #[structopt(short, long, default_value="8,10,11,16,17,18,19,52,54")]
    filter: String,

    /// Add debugging prints for e.g. building color filter.
    #[structopt(long)]
    debug: bool
}

type Color = u8;
type ColorMap = HashMap<String, Color>;
type ColorScheme = Vec<Color>;

fn parse_line(
    line: &str,
    delimiter: &Regex,
    column: u32,
    map: &mut ColorMap,
    color_scheme: &mut ColorScheme) -> Option<Color> {

    let results = delimiter.split(line).collect::<Vec<&str>>();

    if let Some(key) = results.get(column as usize) {
        match map.get(key.clone()) {
            None => {
                let color = color_scheme[0];

                if 1 < color_scheme.len() {
                    map.insert(key.to_string(), color);
                    color_scheme.remove(0);
                }

                Some(color)
            }
            Some(val) => Some(*val)
        }
    } else {
        None
    }
}

fn print_line(line: &String, color: Option<u8>, debug: bool) {
    if let Some(c) = color {
        if debug {
            println!("{}: {}", c, Fixed(c).paint(line));
        } else {
            println!("{}", Fixed(c).paint(line));
        }
    } else {
        println!("{}", line);
    }
}

fn main() {
    let opts = Opts::from_args();

    let mut color_map = ColorMap::new();

    let color_filter = opts.filter
        .split(",")
        .map(|v| v.parse::<u8>().unwrap())
        .collect::<Vec<u8>>();

    let mut color_scheme = (1..255)
        .filter(|e| !color_filter.iter().any(|f| f == e))
        .collect();

    let rex = Regex::new(opts.delimiter.as_str()).unwrap();

    if opts.input == "-" {
        for line in io::stdin().lock().lines() {
            if let Ok(l) = line {
                print_line(&l, parse_line(&l, &rex, opts.column, &mut color_map, &mut color_scheme), opts.debug);
            } else {
                println!("Failed to read line");
            }
        }
    } else {
        if let Ok(file) = File::open(opts.input) {
            let reader = BufReader::new(file);

            for line in reader.lines() {
                if let Ok(l) = line {
                    print_line(&l, parse_line(&l, &rex, opts.column, &mut color_map, &mut color_scheme), opts.debug);
                } else {
                    println!("Failed to read line");
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use regex::Regex;
    use super::*;

    #[test]
    // Make sure that the color scheme is drained, and the color map is expanded
    // when there's no match.
    fn parse_line_with_no_match(){
        let delimiter = Regex::new(r"[ \t]+").unwrap();
        let column = 1;
        let mut color_map = ColorMap::new();
        color_map.insert("keyword".to_string(), 1);
        let mut color_scheme: Vec<Color> = [2, 3, 4].to_vec();

        assert_eq!(parse_line("word key word", &delimiter, column, &mut color_map, &mut color_scheme), Some(2));
        assert_eq!(color_map.len(), 2);
        assert_eq!(color_scheme, [3, 4]);
    }

    #[test]
    // Make sure that the color scheme and color map is unchanged when there's
    // a match.
    fn parse_line_with_match(){
        let delimiter = Regex::new(r"[ \t]+").unwrap();
        let column = 1;
        let mut color_map = ColorMap::new();
        color_map.insert("key".to_string(), 1);
        let mut color_scheme: Vec<Color> = [2, 3, 4].to_vec();

        assert_eq!(parse_line("word key word", &delimiter, column, &mut color_map, &mut color_scheme), Some(1));
        assert_eq!(color_map.len(), 1);
        assert_eq!(color_scheme, [2, 3, 4]);
    }

    #[test]
    // Make sure that the color scheme and color map is unchanged when there's
    // only a single color left.
    fn running_out_of_colors() {
        let delimiter = Regex::new(r"[ \t]+").unwrap();
        let column = 1;
        let mut color_map = ColorMap::new();
        color_map.insert("keyword".to_string(), 1);
        let mut color_scheme: Vec<Color> = [2].to_vec();

        assert_eq!(parse_line("word key word", &delimiter, column, &mut color_map, &mut color_scheme), Some(2));
        assert_eq!(color_map.len(), 1);
        assert_eq!(color_scheme, [2]);
    }

    #[test]
    // Make sure that we handle invalid column gracefully
    fn invalid_column() {
        let delimiter = Regex::new(r"[ \t]+").unwrap();
        let column = 4;
        let mut color_map = ColorMap::new();
        color_map.insert("keyword".to_string(), 1);
        let mut color_scheme: Vec<Color> = [2].to_vec();

        assert_eq!(parse_line("word key word", &delimiter, column, &mut color_map, &mut color_scheme), None);
        assert_eq!(color_map.len(), 1);
        assert_eq!(color_scheme, [2]);
    }
}
