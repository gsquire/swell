#![allow(unstable)]
extern crate hyper;

#[macro_use] extern crate log;

use std::io::BufferedReader;
use std::io::File;
use std::io::fs::PathExtensions;
use std::io::net::ip::Ipv4Addr;

use hyper::Get;
use hyper::header::ContentLength;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { error!("Error: {}", e); return; }
        }
    }}
);

// Send a file as the response, but read it line by line instead of all at
// once to be easier on memory.
fn buffered_file_read(file: File, file_size: u64, mut res: Response) {
    res.headers_mut().set(ContentLength(file_size));
    let mut res = try_return!(res.start());
    let mut reader = BufferedReader::new(file);

    for line in reader.lines() {
        try_return!(res.write_str(line.unwrap().as_slice()));
    }

    try_return!(res.end());
}

fn send_file(path: &str, mut res: Response) {
    let root = "/Users/gsquire/poly/senior_project/html";
    let file_path: Path;
    let file_to_send: File;

    // By default, try and send the index 
    if path == "/" {
        file_path = Path::new(root.to_string() + "/index.html");
    } else {
        file_path = Path::new(root.to_string() + path);
    }

    let file_stat = match file_path.stat() {
        Ok(stat) => stat,
        Err(e) => { panic!("Could not perform stat on file: {}", e); }
    };

    file_to_send = try_return!(File::open(&file_path));

    buffered_file_read(file_to_send, file_stat.size, res);
}

fn base(req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, path.as_slice()) {
            (&Get, _) => {
                send_file(path.as_slice(), res);
                return;
            },
            _ => {
                *res.status_mut() = hyper::NotFound;
                try_return!(res.start().and_then(|res| res.end()));
                return;
            }
        },
        _ => {
            try_return!(res.start().and_then(|res| res.end()));
            return;
        }
    };
}

/// The main method that makes a new hyper Server on port 42007.
/// It starts listening and loops until we send the kill signal.
fn main() {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 42007);
    let mut listener = server.listen(base).unwrap();
    println!("Listening on port 42007...");
    listener.await();
}

#[cfg(test)]
mod tests {
    #[test]
    fn works() {
        assert!(true);
    }
}
