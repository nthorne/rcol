use structopt::StructOpt;
use std::{io, io::BufReader, io::prelude::*};
use std::collections::HashMap;
use regex::Regex;
use ansi_term::Colour::Fixed;
use std::fs::File;
use serde::{Deserialize, Serialize};


mod build_result {
    // Include build script containing e.g. config path.
    include!(concat!(env!("OUT_DIR"), "/build_result.rs"));
}


/// This type represents the configuration file data.
#[derive(Serialize, Deserialize)]
struct Config {
    delimiter: String,
    column: u32,
    filter: String,
}

#[derive(StructOpt, Debug)]
#[structopt(
    author = "Niklas Thorne <notrupertthorne AT gmail>",
    about = build_result::ABOUT_STRING)]
struct Opts {
    /// Input file to colorize. Defaults to stdin.
    #[structopt(name="INPUT")]
    input: Option<String>,

    /// The column delimiter to use. Default config value is "[ \t]+".
    #[structopt(short, long,)]
    delimiter: Option<String>,

    /// Which column to utilize for colorization. Default config value is 0.
    #[structopt(short, long)]
    column: Option<u32>,

    /// List of colors to not use when coloring. Default config value is "8,10,11,16,17,18,19,52,54".
    #[structopt(short, long)]
    filter: Option<String>,

    /// Add debugging prints for e.g. building color filter.
    #[structopt(long)]
    debug: bool
}

/// This type represents the union of the Config type and the Opts type.
struct MergedOpts {
    input: Option<String>,
    delimiter: Regex,
    column: u32,
    filter: String,
    debug: bool
}

type Color = u8;
type ColorMap = HashMap<String, Color>;
type ColorScheme = Vec<Color>;



impl ::std::default::Default for Config {
    /// These are the default configuration values in case of non-existing config file,
    /// which is the case at least at the first start up of the application.
    fn default() -> Self {
        Self {
            delimiter: "[ \t]+".to_string(),
            column: 0,
            filter: "8,10,11,16,17,18,19,52,54".to_string(),
        }
    }
}

impl MergedOpts {
    /// Creates the union of the command line options and
    /// the configuration file values.
    fn new() -> Self {
        let opts = Opts::from_args();
        let config = load_config();

        Self {
            input : opts.input,

            delimiter : if let Some(delimiter) = &opts.delimiter {
                Regex::new(delimiter.as_str()).unwrap()
            } else {
                Regex::new("[ \t]+").unwrap()
            },

            column: if let Some(column) = &opts.column {
                *column
            } else {
                config.column
            },

            filter: if let Some(filter) = &opts.filter {
                filter.to_string()
            } else {
                config.filter
            },

            debug: opts.debug,
        }
    }
}


/// Parses a line, and applies colors from the color scheme, based on
/// which color group the line should belong to. The color group is selected
/// by splitting the line into columns, and using the supplied column id
/// to map the token to a color (or pick the next available color if token
/// has not been encountered yet().
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

/// Prints a line on stdout using the selected color.
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

/// Loads the configuration file, based on the cargo application name. If the
/// file does not exist, a default one will be created based on the default
/// values of the Config type.
fn load_config() -> Config {
    let name = option_env!("CARGO_PKG_NAME").unwrap();
    return confy::load(name).unwrap();
}


fn main() {
    let opts = MergedOpts::new();

    let mut color_map = ColorMap::new();

    let color_filter = opts.filter
        .split(",")
        .map(|v| v.parse::<u8>().unwrap())
        .collect::<Vec<u8>>();

    let mut color_scheme = (1..255)
        .filter(|e| !color_filter.iter().any(|f| f == e))
        .collect();

    if let Some(file_name) = opts.input {
        if let Ok(file) = File::open(file_name) {
            let reader = BufReader::new(file);

            for line in reader.lines() {
                if let Ok(l) = line {
                    print_line(&l, parse_line(&l, &opts.delimiter, opts.column, &mut color_map, &mut color_scheme), opts.debug);
                } else {
                    println!("Failed to read line");
                }
            }
        }
    } else {
        for line in io::stdin().lock().lines() {
            if let Ok(l) = line {
                print_line(&l, parse_line(&l, &opts.delimiter, opts.column, &mut color_map, &mut color_scheme), opts.debug);
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
