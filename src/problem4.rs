use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};

const ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 1337;

lazy_static::lazy_static! {
    static ref STATE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug)]
enum Request {
    Insert { key: String, value: String },
    Query(String),
    Version,
}

fn main() -> Result<(), std::io::Error> {
    let socket = UdpSocket::bind(SocketAddrV4::new(ADDR, PORT))?;
    println!("Problem 4 listening on UDP {:?}:{:?}", ADDR, PORT);

    while let Ok((addr, message)) = read_udp_package(&socket) {
        println!("Received packet from {:?}: {}", addr, &message);
        let request = parse_message(&message);
        println!("Parsed request: {:?}", request);
        match request {
            Request::Insert { key, value } => {
                let mut state = STATE.lock().unwrap();
                state.insert(key, value);
            }
            Request::Query(key) => {
                let state = STATE.lock().unwrap();
                let value = state.get(&key).cloned().unwrap_or_default();
                let response = format!("{}={}", key, value);
                socket.send_to(response.as_bytes(), addr)?;
            }
            Request::Version => {
                socket.send_to("version=Ken's Key-Value Store 1.0".as_bytes(), addr)?;
            }
        }
    }

    Ok(())
}

fn read_udp_package(socket: &UdpSocket) -> Result<(SocketAddr, String), std::io::Error> {
    let mut buf = vec![0; 65507];
    let (size, addr) = socket.recv_from(&mut buf)?;
    let message = String::from_utf8_lossy(&buf[..size]).to_string();
    Ok((addr, message))
}

fn parse_message(message: &String) -> Request {
    let splits = message.splitn(2, "=").collect::<Vec<_>>();
    match splits.as_slice() {
        [key, value] => Request::Insert {
            key: key.to_string(),
            value: value.to_string(),
        },
        _ => {
            if message == "version" {
                Request::Version
            } else {
                Request::Query(message.clone())
            }
        }
    }
}
