use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::thread::Thread;

fn handle_client(mut stream: TcpStream) {
    let resp = b"HTTP/1.1 200 OK\r\nContent-Type: text/html;\r\n\r\nHello!\n";
    match stream.write(resp) {
        Err(e) => { println!("Error writing response: {}", e); }
        Ok(_) => {}
    }
}

fn main() {
    let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 42007 };
    let listener = TcpListener::bind(addr).unwrap();

    let mut acceptor = listener.listen();
    println!("Listening on port {}", addr.port);

    for stream in acceptor.incoming() {
        Thread::spawn(move || {
            handle_client(stream.unwrap())
        });
    }

    drop(acceptor);
}
