#![allow(unstable)]
extern crate hyper;

#[macro_use] extern crate log;

use std::io::BufferedReader;
use std::io::File;
use std::io::fs::PathExtensions;
use std::io::net::ip::Ipv4Addr;
use std::vec::Vec;

use hyper::Get;
use hyper::header::ContentLength;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
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
fn buffered_file_read(file: File, res: Response) {
    let mut response = try_return!(res.start());
    let mut reader = BufferedReader::new(file);

    for line in reader.lines() {
        try_return!(response.write_str(line.unwrap().as_slice()));
    }

    try_return!(response.end());
}

// Given a path extension, return back the proper MIME type.
// The default is text/plain.
fn get_content_type(extension: &str) -> Mime {
    let opts = Vec::new(); // This is empty as we don't use options.

    match extension {
        "html" => Mime(TopLevel::Text, SubLevel::Html, opts),
        "css" => Mime(TopLevel::Text, SubLevel::Css, opts),
        "js" => Mime(TopLevel::Application, SubLevel::Javascript, opts),
        "jpeg" | "jpg" => Mime(TopLevel::Image, SubLevel::Jpeg, opts),
        "png" => Mime(TopLevel::Image, SubLevel::Png, opts),
        "gif" => Mime(TopLevel::Image, SubLevel::Gif, opts),
        _ => Mime(TopLevel::Text, SubLevel::Plain, opts)
    }
}

// Write a static file as the response back in a GET request.
fn send_file(path: &str, mut res: Response) {
    let root = "/Users/gsquire/poly/senior_project/html";
    let file_path: Path;
    let file_to_send: File;

    // Let everyone know what cool server sent them this response.
    res.headers_mut().set(hyper::header::Server("swell".to_string()));

    // By default, try and send the index 
    if path == "/" {
        file_path = Path::new(root.to_string() + "/index.html");
    } else {
        file_path = Path::new(root.to_string() + path);
    }

    // Get the file extension so we can set the MIME type.
    let ext = file_path.extension().unwrap(); 
    let ext_str = std::str::from_utf8(ext).unwrap();

    // We send a 404 response in the error case here, 203 is the size of
    // the file since it will not change.
    let file_stat = match file_path.stat() {
        Ok(stat) => stat,
        Err(e) => {
            println!("Could not perform stat on file: {}", e);
            *res.status_mut() = hyper::NotFound;
            let error_file = Path::new(root.to_string() + "/404.html");
            file_to_send = try_return!(File::open(&error_file));

            res.headers_mut().set(ContentLength(203)); // This is constant.
            res.headers_mut().set(ContentType(get_content_type("html".as_slice())));
            buffered_file_read(file_to_send, res);
            return;
        }
    };

    file_to_send = try_return!(File::open(&file_path));

    res.headers_mut().set(ContentType(get_content_type(ext_str)));
    res.headers_mut().set(ContentLength(file_stat.size));

    buffered_file_read(file_to_send, res);
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
    const NUM_THREADS: usize = 64;
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 42007);
    let mut listener = server.listen_threads(base, NUM_THREADS).unwrap();
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
