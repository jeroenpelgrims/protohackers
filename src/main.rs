use std::io::{Bytes, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::Shutdown;
use std::thread;
use std::net::{TcpListener, TcpStream};

const ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 1337;

fn main() {
    let listener = TcpListener::bind(SocketAddrV4::new(ADDR, PORT)).unwrap();

    println!("Listening on TCP {:?}:{:?}", ADDR, PORT);

    for stream in listener.incoming() {
        match stream {
            Err(err) => println!("Connection failed: {:?}", err),
            Ok(mut stream) => {
                println!("Connection from: {:?}", stream.peer_addr().unwrap());
                thread::spawn(move|| { handle_client(stream) });
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8;60]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(0) => {
            stream.shutdown(Shutdown::Both).unwrap();
            true
        },
        Ok(size) => {
            // echo everything!
            stream.write(&data[0..size]).unwrap();
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}
