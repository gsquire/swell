#![feature(old_path)]
#![feature(old_io)]
#![feature(core)]
#![feature(rustc_private)]

extern crate hyper;
extern crate swell;
extern crate toml;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use std::old_io::BufferedReader;
use std::old_io::File;
use std::old_io::fs::PathExtensions;
use std::old_io::net::ip::Ipv4Addr;
use std::vec::Vec;

use hyper::Get;
use hyper::header::ContentLength;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

// Allows for dynamic static variable creation at runtime.
lazy_static! {
    static ref config: toml::Value = swell::config::parse_config();
}

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { error!("Error: {}", e); return; }
        }
    }}
);

// Send a file as the response, reading all of the bytes into a buffer and
// then sending that back in the response.
fn buffered_file_read(file: File, res: Response) {
    let mut response = try_return!(res.start());
    let mut reader = BufferedReader::new(file);

    let bytes = reader.read_to_end().unwrap();
    try_return!(response.write_all(bytes.as_slice()));

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
    let root = config.lookup("server.document_root").unwrap().as_str().unwrap();
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
    let ext = match file_path.extension() {
        Some(v) => v,
        None => b"None"
    };
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
            res.headers_mut().set(
                ContentType(get_content_type("html".as_slice())));
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

/// The main method that makes a new hyper Server on the port specified in the
/// configuration file.
/// It starts listening and loops until we send the kill signal.
fn main() {
    let num_threads: usize =
        config.lookup("server.num_threads").unwrap().as_integer().unwrap() as
        usize;
    let port: u16 =
        config.lookup("server.port").unwrap().as_integer().unwrap() as u16;
    let server = Server::http(Ipv4Addr(0, 0, 0, 0), port);
    let mut _listener = server.listen_threads(base, num_threads).unwrap();

    println!("Listening on port 42007...");
}

#[cfg(test)]
mod tests {
    use super::get_content_type;

    use hyper::mime::{Mime, TopLevel, SubLevel};

    #[test]
    fn testing_check() {
        assert!(true);
    }

    #[test]
    fn check_jpeg_content_type() {
        let jpeg_mime = Mime(TopLevel::Image, SubLevel::Jpeg, Vec::new());
        assert_eq!(jpeg_mime, get_content_type("jpeg".as_slice()));
        assert_eq!(jpeg_mime, get_content_type("jpg".as_slice()));
    }

    #[test]
    fn check_js_content_type() {
        let js_mime = Mime(TopLevel::Application, SubLevel::Javascript, Vec::new());
        assert_eq!(js_mime, get_content_type("js".as_slice()));
    }
}
