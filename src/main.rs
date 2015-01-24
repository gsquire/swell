#![allow(unstable)]
extern crate hyper;

#[macro_use] extern crate log;

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

fn default(req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, path.as_slice()) {
            (&Get, _) => {
                let out = b"Test\n";
                println!("Path is: {}", path.as_slice());

                res.headers_mut().set(ContentLength(out.len() as u64));
                let mut res = try_return!(res.start());
                try_return!(res.write(out));
                try_return!(res.end());
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
    let mut listener = server.listen(default).unwrap();
    println!("Listening on port 42007...");
    listener.await();
}
