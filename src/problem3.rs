use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind};
use std::io::{BufReader, Read, Write};
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use regex::Regex;

const ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 1337;

struct ConnectionInfo {
    username: String,
    stream: TcpStream,
}

lazy_static::lazy_static! {
    static ref USERS: Arc<Mutex<HashMap<SocketAddr, ConnectionInfo>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(SocketAddrV4::new(ADDR, PORT))?;
    println!("Problem 3 listening on TCP {:?}:{:?}", ADDR, PORT);

    for stream in listener.incoming() {
        match stream {
            Err(err) => println!("Connection failed: {:?}", err),
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream));
            }
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) {
    let peer_addr = stream
        .peer_addr()
        .expect("Error getting peer_addr from stream");

    match join(peer_addr, &stream) {
        Ok(()) => (),
        Err(_) => {
            println!("Failed to join: {:?}", peer_addr.clone());
            disconnect(&stream);
            return;
        }
    }

    loop {
        let Ok(stream) = get_stream(&peer_addr) else {
            break;
        };
        let Ok(message) = read_message(&stream) else {
            break;
        };
        if !message.is_empty() {
            let Ok(()) = broadcast_message(&peer_addr, &message, false) else {
                break;
            };
        }
    }

    part(&peer_addr);
}

fn join(peer_addr: SocketAddr, stream: &TcpStream) -> Result<(), std::io::Error> {
    println!("New connection: {:?}", peer_addr);

    send_message(&stream, "Welcome to budgetchat! What shall I call you?")?;
    let username = read_message(stream)?;

    let username_pattern = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
    if username.trim().is_empty() || !username_pattern.is_match(&username) {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Username cannot be empty or contain invalid characters",
        ));
    }

    println!("User {} joined from {:?}", username, peer_addr);

    let connected_usernames = {
        let users = USERS.lock().unwrap();
        users
            .values()
            .map(|u| u.username.clone())
            .collect::<Vec<_>>()
    };

    broadcast_message(
        &peer_addr,
        &format!("{} has entered the room", username),
        true,
    )?;
    send_message(
        &stream,
        &format!("* The room contains: {}", connected_usernames.join(", ")),
    )?;

    {
        let mut users = USERS.lock().unwrap();
        users.insert(
            peer_addr,
            ConnectionInfo {
                username: username.clone(),
                stream: stream.try_clone()?,
            },
        );
    }

    Ok(())
}

fn disconnect(stream: &TcpStream) {
    let result = stream.shutdown(Shutdown::Both);
    match result {
        Ok(()) => println!("Disconnected: {:?}", stream.peer_addr()),
        Err(err) => println!("Error disconnecting: {:?}", err),
    }
    let mut users = USERS.lock().unwrap();
    users.remove(&stream.peer_addr().unwrap());
}

fn part(peer_addr: &SocketAddr) {
    let Ok(stream) = get_stream(peer_addr) else {
        println!("Failed to get stream for {:?}", peer_addr);
        return;
    };

    let username = get_username(peer_addr).unwrap_or_default();
    let part_message = format!("{} has left the room", username);
    broadcast_message(peer_addr, &part_message, true).unwrap();

    disconnect(&stream);
}

fn get_stream(peer_addr: &SocketAddr) -> Result<TcpStream, std::io::Error> {
    let users = USERS.lock().unwrap();

    let user_info = users.get(&peer_addr).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("User with address {} not found", peer_addr),
        )
    })?;

    user_info.stream.try_clone()
}

fn send_message(mut stream: &TcpStream, message: &str) -> Result<(), std::io::Error> {
    let result = stream.write_all(format!("{message}\n").as_bytes());
    if let Err(err) = result {
        part(&stream.peer_addr().unwrap());
        return Err(err);
    }
    println!("Sent message: {}", message);
    Ok(())
}

fn read_message(stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut reader = BufReader::new(stream);
    let mut message = String::new();
    match reader.read_line(&mut message) {
        Ok(0) => Err(std::io::Error::new(
            ErrorKind::UnexpectedEof,
            "Connection closed",
        )),
        Ok(_) => Ok(message.trim().to_string()),
        Err(e) => Err(e),
    }
}

fn broadcast_message(
    sender_addr: &SocketAddr,
    message: &str,
    system_message: bool,
) -> Result<(), std::io::Error> {
    let (sender_username, streams) = {
        let users = USERS.lock().unwrap();
        let sender_username = users
            .get(sender_addr)
            .map(|u| u.username.clone())
            .unwrap_or_default();
        let streams = users
            .iter()
            .filter(|(addr, _)| !(*addr).eq(sender_addr))
            .map(|(_, user)| user.stream.try_clone())
            .collect::<Result<Vec<_>, _>>()?;
        (sender_username, streams)
    };

    let full_message = if system_message {
        format!("* {}", message)
    } else {
        format!("[{}] {}", sender_username, message)
    };
    for stream in streams {
        send_message(&stream, &full_message)?;
    }
    Ok(())
}

fn get_username(peer_addr: &SocketAddr) -> Option<String> {
    let users = USERS.lock().unwrap();
    users.get(peer_addr).map(|u| u.username.clone())
}
