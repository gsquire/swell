extern crate hyper;
extern crate swell;
extern crate toml;
extern crate time;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Write};
use std::path::Path;
use std::vec::Vec;

use hyper::Get;
use hyper::header::ContentLength;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

// Allows for dynamic static variable creation at runtime.
lazy_static! {
    static ref args: Vec<String> = env::args().collect();
    //static ref config: toml::Table =
        //swell::config::parse_config(&args[1].as_ref());
    static ref config: toml::Table =
        swell::config::parse_config("/Users/gsquire/poly/senior_project/swell_config.toml");
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
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    try_return!(response.write_all(&buffer.as_ref()));
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

/// This function simply returns a string representation of the current time.
/// It is used for logging requests.
fn cur_time_string() -> String {
    let cur_time = time::now();
    let timestamp = time::strftime("%F %T", &cur_time).unwrap();

    timestamp
}

/// This is a generic method that helps us write a file (resource) back to a
/// client. It checks for the default index and builds a path accordingly. It
/// is also responsible for getting the resources extension so that it can set
/// the MIME type accordingly. By default it gets the file size as well to set
/// Content-Length for the browser's response headers. It handles a 404 case by
/// responding with a generic 404 file.
fn send_file(req: &Request, path: &str, mut res: Response) {
    //let root = config.get(&"server.document_root".to_string()).unwrap().as_str().unwrap();
    let root = "/Users/gsquire/poly/senior_project/html";
    let root_path = Path::new(root);
    let file_to_send: File;

    // Let everyone know what cool server sent them this response.
    res.headers_mut().set(hyper::header::Server("swell".to_string()));

    // By default, try and send the index. 
    let new_path = match path {
        "/" => root_path.join("/index.html"),
        _ => root_path.join(path)
    };

    // Get the file extension so we can set the MIME type.
    let ext = match new_path.as_path().extension() {
        Some(v) => v,
        None => OsStr::new("None")
    };
    let ext_str = ext.to_str().unwrap();

    // We send a 404 response in the error case here, 203 is the size of
    // the file since it will not change.
    file_to_send = try_return!(File::open(&new_path));

    let file_metadata = match file_to_send.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Could not perform stat on file: {}", e);
            info!("{} {} {} 404 {}",
                  cur_time_string(), req.method, path, req.remote_addr);

            *res.status_mut() = hyper::NotFound;
            let joined_path = root_path.join("/404.html");
            let file = try_return!(File::open(&joined_path.as_path()));

            res.headers_mut().set(ContentLength(203)); // This is constant.
            res.headers_mut().set(
                ContentType(get_content_type("html".as_ref())));
            buffered_file_read(file, res);
            return;
        }
    };

    res.headers_mut().set(ContentType(get_content_type(ext_str)));
    res.headers_mut().set(ContentLength(file_metadata.len()));

    info!("{} {} {} 200 {}",
          cur_time_string(), req.method, path, req.remote_addr);
    buffered_file_read(file_to_send, res);
}

/// This is the entry method for the Hyper server. It unwraps a request
/// structure and ensures that we handle each one correctly. The response
/// object helps us set headers and other HTTP information safely.
fn base(req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, path) {
            (&Get, _) => {
                send_file(&req, path.as_ref(), res);
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
        config.get(&"server.num_threads".to_string()).unwrap().as_integer().unwrap() as
        usize;
    let port: u16 =
        config.get(&"server.port".to_string()).unwrap().as_integer().unwrap() as u16;
    let server = Server::http(base);
    let mut _listener = server.listen_threads("0.0.0.0:42007", num_threads).unwrap();

    // Initialize our logging library to standard out.
    swell::logger::init();

    info!("Listening on port 42007...");
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
