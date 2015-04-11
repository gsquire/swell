extern crate toml;

use std::convert::AsRef;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::string::String;

/// Given a TOML configuration file, parse it to use throughout the
/// life of the server. It returns what is basically a wrapped hash table with
/// the values that are specified in the Swell configuration.
pub fn parse_config(config_path: &str) -> toml::Table {
    let path = Path::new(config_path);
    let mut config_file = File::open(&path).unwrap();
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer).unwrap();
    let mut parser = toml::Parser::new(buffer.as_ref());

    match parser.parse() {
        Some(value) => value,
        None => {
            panic!("Parser errors: {:?}", parser.errors);
        }
    }
}
