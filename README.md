swell
=====

A web server in Rust.

This is my senior project at Cal Poly. It will be under constant construction
until June when I graduate. I will do my best to keep up with the Rust releases.
I am no expert and this is a huge learning experience for me and should not be
used in production in any form.

The name is inspired from my love of the ocean and surfing.

Currently, I build the project using 1.0.0-beta. Once the stable version is
released I will use it, but I am sticking with beta now.

To run the server:
```sh
cargo build --release

cargo run PATH (PATH is the file path to the configuration file used)
```

Here is an example configuration file:
```toml
[server]

document_root = "/Users/gsquire/poly/senior_project/html"
num_threads = 16
port = 42007
```
The file format is TOML and the specification for that format can be
found [here](https://github.com/toml-lang/toml).

Current configuration options:
* document_root is the root directory from which to serve files from
* num_threads is the number of threads the server will use
* port is the port on which it will serve files
