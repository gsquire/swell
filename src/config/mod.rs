#![crate_id="config"]
#![crate_type="lib"]

extern crate toml;

use std::old_io::File;

/// Given a TOML configuration file, parse it to use throughout the
/// life of the server. It returns what is basically a wrapped hash table with
/// the values that are specified in the Swell configuration.
pub fn parse_config() -> toml::Value {
    let path =
        Path::new("/Users/gsquire/poly/senior_project/swell_config.toml");
    let mut config_file = File::open(&path);
    let contents = config_file.read_to_string().unwrap();

    contents.as_slice().parse().unwrap()
}
