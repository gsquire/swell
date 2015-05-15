swell
=====

A web server in Rust.

This is my senior project at Cal Poly. It will be under constant construction
until June when I graduate. I will do my best to keep up with the Rust releases.
I am no expert and this is a huge learning experience for me and should not be
used in production in any form.

The name is inspired from my love of the ocean and surfing.

The project compiles on Rust 1.0.0 stable, which I will continue to track.

You can define dynamic routes by using the endpoints and it's associated port
as seen in the example below. This is a way to add some REST functionality
to the server. Is it ideal? Probably not. But you can use any other process
on the machine, open a port, and swell will send requests to it.

Thus, it is language agnostic. I hope that is attractive at least.

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
endpoints = ["/test"]
endpoint_port = 3000
```
The file format is TOML and the specification for that format can be
found [here](https://github.com/toml-lang/toml).

Current configuration options:
* document_root is the root directory from which to serve files from
* num_threads is the number of threads the server will use
* port is the port on which it will serve files
* endpoints is the set of dynamic routes you wish to serve on the specified
port.
* endpoint_port is the port to send requests to for the dynamic routes
