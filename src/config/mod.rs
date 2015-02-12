extern crate toml;

use std::old_io::File;

// Parse the config file and return the table back to the server to use
// throughout the program.
pub fn parse_config() -> toml::Value {
    let path =
        Path::new("/Users/gsquire/poly/senior_project/swell_config.toml");
    let mut config_file = File::open(&path);
    let contents = config_file.read_to_string().unwrap();

    contents.as_slice().parse().unwrap()
}
