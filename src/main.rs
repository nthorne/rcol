use clap::Clap;
use std::{io, io::prelude::*};
use std::collections::HashMap;
use regex::Regex;
use ansi_term::Colour::Fixed;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Niklas Thorne <notrupertthorne AT gmail>")]
struct Opts {
    #[clap(name="INPUT", default_value="-", about="Input file to colorize. Defaults to stdin.")]
    input: String,
    #[clap(short, long, default_value="[ \t]+", about="The column delimiter to use.")]
    delimiter: String,
    #[clap(short, long, default_value="0", about="Which column to utilize for colorization.")]
    column: u32,
    #[clap(short, long, default_value="8,10,11,16,17,18,19,52,54", about="Comma separated list of colors to filter out.")]
    filter: String,
    #[clap(long, about="Add debugging prints for e.g. building color filter.")]
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

fn main() {
    let opts = Opts::parse();

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
                if let Some(color) = parse_line(&l, &rex, opts.column, &mut color_map, &mut color_scheme) {
                    if opts.debug {
                        println!("{}: {}", color, Fixed(color).paint(l));
                    } else {
                        println!("{}", Fixed(color).paint(l));
                    }
                } else {
                    println!("{}", l);
                }
            } else {
                println!("Failed to read line");
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
