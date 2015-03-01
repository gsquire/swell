#![crate_id="swell"]
#![crate_type="bin"]

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

/// A macro I borrowed from Hyper to help unwrap a Result<T> enum.
macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { error!("Error: {}", e); return; }
        }
    }}
);

/// Take a file object and create a buffered reader to read from and send back
/// to the client. It ends the Hyper response as well since the resource will
/// have been successfully returned.
fn buffered_file_read(file: File, res: Response) {
    let mut response = try_return!(res.start());
    let mut reader = BufferedReader::new(file);

    let bytes = reader.read_to_end().unwrap();
    try_return!(response.write_all(bytes.as_slice()));

    try_return!(response.end());
}

/// This method takes a file path extension and returns a Hyper Mime structure.
/// It gives us a type-safe structure that Hyper will correctly encode in the
/// response. The options are empty for now, as this rarely needs to be set.
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

/// This is a generic method that helps us write a file (resource) back to a
/// client. It checks for the default index and builds a path accordingly. It
/// is also responsible for getting the resources extension so that it can set
/// the MIME type accordingly. By default it gets the file size as well to set
/// Content-Length for the browser's response headers. It handles a 404 case by
/// responding with a generic 404 file.
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

/// This is the entry method for the Hyper server. It unwraps a request
/// structure and ensures that we handle each one correctly. The response
/// object helps us set headers and other HTTP information safely.
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
