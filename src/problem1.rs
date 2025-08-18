use std::io::{Read, Write};
use std::net::Shutdown;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{TcpListener, TcpStream};
use std::thread;

const ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 1337;

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(SocketAddrV4::new(ADDR, PORT))?;

    println!("Problem 1 listening on TCP {:?}:{:?}", ADDR, PORT);

    for stream in listener.incoming() {
        match stream {
            Err(err) => println!("Connection failed: {:?}", err),
            Ok(stream) => {
                println!("Connection from: {:?}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    let peer_addr = stream.peer_addr().unwrap();
                    let result = handle_client(stream);
                    if result.is_err() {
                        println!("An error occurred with peer: {}", peer_addr);
                    }
                });
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    Ok(())
}
