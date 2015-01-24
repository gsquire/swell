#![allow(unstable)]
extern crate hyper;

#[macro_use] extern crate log;

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

fn send_file(path: &str, mut res: Response) {
    let root = "/Users/gsquire/poly/senior_project/html";
    let out: String;

    if path == "/" {
        out = try_return!(File::open(&Path::new(root.to_string() + "/index.html")).read_to_string());
    } else {
        // Let us check if the path actually exists.
        let path = Path::new(root.to_string() + path);
        if path.exists() {
            out = try_return!(File::open(&path).read_to_string());
        } else {
            // Return a 404.
            *res.status_mut() = hyper::NotFound;
            try_return!(res.start().and_then(|res| res.end()));
            return;
        }
    }

    res.headers_mut().set(ContentLength(out.len() as u64));
    let mut res = try_return!(res.start());
    try_return!(res.write_str(out.as_slice()));
    try_return!(res.end());
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

fn main() {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 42007);
    let mut listener = server.listen(base).unwrap();
    println!("Listening on port 42007...");
    listener.await();
}
