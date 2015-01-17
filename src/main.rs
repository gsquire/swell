extern crate hyper;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

use hyper::http::read_request_line;

fn handle_client(mut stream: TcpStream) {
    match stream.read_to_string() {
        Err(e) => { println!("error reading request: {}", e); }
        Ok(req) => { println!("the request is: {}", req); }
    }
    let resp = b"HTTP/1.1 200 OK\r\nContent-Type: text/html;\r\n\r\nHello!\n";
    match stream.write(resp) {
        Err(e) => { println!("Error writing response: {}", e); }
        Ok(_) => {}
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:42007");

    let mut acceptor = listener.listen();

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => { println!("Error in stream: {}", e); }
            Ok(stream) => { handle_client(stream) }
        }
    }

    drop(acceptor);
}
