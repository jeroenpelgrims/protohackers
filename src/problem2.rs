use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 1337;

type Price = i32;
type Timestamp = i32;
type PriceMap = HashMap<Timestamp, Price>;
type GlobalState = Arc<Mutex<HashMap<SocketAddr, PriceMap>>>;

lazy_static::lazy_static! {
    static ref GLOBAL_STATE: GlobalState = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug)]
enum Message {
    InsertMessage {
        timestamp: Timestamp,
        price: Price,
    },
    QueryMessage {
        mintime: Timestamp,
        maxtime: Timestamp,
    },
}

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(SocketAddrV4::new(ADDR, PORT))?;

    println!("Problem 2 listening on TCP {:?}:{:?}", ADDR, PORT);

    for stream in listener.incoming() {
        match stream {
            Err(err) => println!("Connection failed: {:?}", err),
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from: {:?}", peer_addr);

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buffer = [0u8; 9];
    while reader.read_exact(&mut buffer).is_ok() {
        let message = parse_message(buffer);
        match message {
            None => {
                println!("Failed to parse message: {:?}", buffer);
            }
            Some(Message::InsertMessage { timestamp, price }) => {
                // println!("Insert {:?}", message);
                handle_insert(peer_addr, timestamp, price);
            }
            Some(Message::QueryMessage { mintime, maxtime }) => {
                // println!("Query {:?}", message);
                let price = handle_query(peer_addr, mintime, maxtime).unwrap_or(0);
                let price_bytes = (price as i32).to_be_bytes();
                stream.write_all(&price_bytes)?;
            }
        }
    }

    stream.shutdown(Shutdown::Both)
}

fn parse_message(buffer: [u8; 9]) -> Option<Message> {
    if buffer.len() != 9 {
        return None;
    }

    match buffer.get(0) {
        Some(&b'I') => {
            let timestamp = i32::from_be_bytes(buffer[1..5].try_into().unwrap());
            let price = i32::from_be_bytes(buffer[5..9].try_into().unwrap());
            Some(Message::InsertMessage { timestamp, price })
        }
        Some(&b'Q') => {
            let mintime = i32::from_be_bytes(buffer[1..5].try_into().unwrap());
            let maxtime = i32::from_be_bytes(buffer[5..9].try_into().unwrap());
            Some(Message::QueryMessage { mintime, maxtime })
        }
        _ => None,
    }
}

fn handle_insert(peer_addr: SocketAddr, timestamp: Timestamp, price: Price) {
    let mut global_state = GLOBAL_STATE.lock().unwrap();
    let price_map = global_state.entry(peer_addr).or_insert_with(HashMap::new);
    price_map.insert(timestamp, price);
}

fn handle_query(peer_addr: SocketAddr, mintime: Timestamp, maxtime: Timestamp) -> Option<Price> {
    let global_state = GLOBAL_STATE.lock().unwrap();
    let price_map = global_state.get(&peer_addr)?;

    let valid_timestamps = price_map
        .keys()
        .filter(|&&timestamp| timestamp >= mintime && timestamp <= maxtime)
        .cloned()
        .collect::<Vec<i32>>();
    let prices = valid_timestamps
        .iter()
        .map(|timestamp| {
            price_map
                .get(timestamp)
                .expect("Given timestamp should exist")
        })
        .cloned()
        .collect::<Vec<Price>>();

    if prices.is_empty() {
        None
    } else {
        let sum: i64 = prices.iter().map(|&price| price as i64).sum();
        let mean = sum / prices.len() as i64;
        Some(mean as Price)
    }
}
