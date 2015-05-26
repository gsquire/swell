extern crate flate2;
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
use std::path::PathBuf;
use std::vec::Vec;

use hyper::Client;
use hyper::Get;
use hyper::header::Encoding;
use hyper::header::{AcceptEncoding, Connection, ContentLength, ContentType, ContentEncoding};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

use flate2::Compression;
use flate2::write::GzEncoder;

// Allows for dynamic static variable creation at runtime.
lazy_static! {
    static ref ARGS: Vec<String> = env::args().collect();
    static ref CONFIG: toml::Table =
        swell::config::parse_config(&ARGS[1].as_ref());
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

/// A GZIP encoding writer for requests that accept it as a means of
/// compression.
fn gzip_encoded_read(file: File, mut res: Response) {
    let mut ce_vec = Vec::new();
    ce_vec.push(Encoding::Gzip);
    res.headers_mut().set(ContentEncoding(ce_vec));

    let mut response = try_return!(res.start());
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    let mut gzip_enc = GzEncoder::new(Vec::new(), Compression::Best);
    let _ = gzip_enc.write_all(&buffer.as_ref());
    let compressed_bytes = gzip_enc.finish();

    try_return!(response.write_all(compressed_bytes.unwrap().as_ref()));
    try_return!(response.end());
}

fn accepts_gzip_encoding(ae: &AcceptEncoding) -> bool {
    for encoding in ae.iter() {
        if encoding.item == Encoding::Gzip {
            return true;
        }
    }
    false
}

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
        "mp4" => Mime(TopLevel::Video, SubLevel::Ext("mp4".to_string()), opts),
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
    let server_table = CONFIG.get(&"server".to_string()).unwrap();
    let root = server_table.lookup("document_root").unwrap().as_str().unwrap();
    let root_path = Path::new(root);
    let mut file_reading_path = PathBuf::from(root_path);

    // Let everyone know what cool server sent them this response.
    res.headers_mut().set(hyper::header::Server("swell".to_string()));

    // By default, try and send the index. 
    let _ = match path {
        "/" => file_reading_path.push("index.html"),
        _ => {
            // We have to trim the root suffix to return the entire path.
            // See the PathBuf documentation for push.
            let trim = path.trim_left_matches('/');
            file_reading_path.push(trim);
        }
    };

    // Get the file extension so we can set the MIME type.
    let ext = match file_reading_path.as_path().extension() {
        Some(v) => v,
        None => OsStr::new("None")
    };
    let ext_str = ext.to_str().unwrap();

    // We send a 404 response in the error case here, 203 is the size of
    // the file since it will not change.
    let file_to_send = match File::open(&file_reading_path) {
        Ok(file_to_send) => file_to_send,
        Err(e) => {
            error!("Could not perform stat on file: {}", e);
            info!("{} {} {} 404 {}",
                  cur_time_string(), req.method, path, req.remote_addr);

            *res.status_mut() = hyper::NotFound;
            let mut error_file = PathBuf::from(root_path);
            error_file.push("404.html");
            let file = try_return!(File::open(&error_file.as_path()));

            res.headers_mut().set(ContentLength(203)); // This is constant.
            res.headers_mut().set(
                ContentType(get_content_type("html".as_ref())));
            buffered_file_read(file, res);
            return;
        }
    };

    info!("{} {} {} 200 {}",
          cur_time_string(), req.method, path, req.remote_addr);

    // We know the file exists if we just opened it.
    let file_metadata = file_to_send.metadata().unwrap();

    res.headers_mut().set(ContentType(get_content_type(ext_str)));

    let accept_encoding = req.headers.get::<AcceptEncoding>();
    match accept_encoding {
        Some(ae) => {
            match accepts_gzip_encoding(ae) {
                true => { gzip_encoded_read(file_to_send, res); },
                // GZIP not supported, send normally.
                false => {
                    res.headers_mut().set(ContentLength(file_metadata.len()));
                    buffered_file_read(file_to_send, res);
                }
            }
        },
        // No encoding accepted, so send normally.
        None => {
            res.headers_mut().set(ContentLength(file_metadata.len()));
            buffered_file_read(file_to_send, res);
        }
    }
}

/// Check if the route request is a dynamic route.
fn is_dynamic_route(route: &str, endpoints: &[toml::Value]) -> bool {
    for r in endpoints.iter() {
        let dyn_route = r.as_str().unwrap();
        if route == dyn_route {
            return true;
        }
    }
    false
}

/// This is our function to handle dynamic requests sent to the port that
/// the user specified.
fn handle_dynamic(req: &Request, route: &str, res: Response) {
    let mut response = try_return!(res.start());
    let dynamic_table = CONFIG.get(&"server".to_string()).unwrap();
    let endpoint_port = dynamic_table.lookup("endpoint_port").unwrap().as_integer().unwrap();
    let url = format!("http://localhost:{}{}", endpoint_port, route);
    let url_str: &str = url.as_ref();
    let mut client = Client::new();
    let mut dyn_res = client.get(url_str).header(Connection::close()).send().unwrap();
    let mut res_string = String::new();

    info!("{} {} {} 200 {}",
          cur_time_string(), req.method, route, req.remote_addr);
    dyn_res.read_to_string(&mut res_string).unwrap();
    try_return!(response.write_all(&res_string.as_bytes()));
    try_return!(response.end());
}

/// This is the entry method for the Hyper server. It unwraps a request
/// structure and ensures that we handle each one correctly. The response
/// object helps us set headers and other HTTP information safely.
fn base(req: Request, mut res: Response) {
    let dynamic_table = CONFIG.get(&"server".to_string()).unwrap();
    let endpoints = dynamic_table.lookup("endpoints").unwrap().as_slice().unwrap();

    match req.uri {
        AbsolutePath(ref path) => match (&req.method, path) {
            (&Get, _) => {
                match is_dynamic_route(path.as_ref(), endpoints) {
                    true => handle_dynamic(&req, path.as_ref(), res),
                    false => send_file(&req, path.as_ref(), res)
                }
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
    let server_table = CONFIG.get(&"server".to_string()).unwrap();
    let num_threads: usize = server_table.lookup("num_threads").unwrap().as_integer().unwrap() as
        usize;
    let port: u16 = server_table.lookup("port").unwrap().as_integer().unwrap() as u16;
    let server = Server::http(base);
    let mut _listener = server.listen_threads(("0.0.0.0", port), num_threads).unwrap();

    // Initialize our logging library to standard out.
    let _logger_error = swell::logger::init();

    info!("Listening on port {}...", port);
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
