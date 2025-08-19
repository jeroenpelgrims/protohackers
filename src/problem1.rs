use num_prime::nt_funcs::is_prime64;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4};
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
                thread::spawn(move || handle_client(stream));
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct PrimeRequest {
    method: String,
    number: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrimeResponse {
    method: String,
    prime: bool,
}

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from: {:?}", peer_addr);

    let reader = BufReader::new(stream.try_clone()?);
    for request in reader.lines() {
        let response = handle_request(request);
        match response {
            Ok(resp) => {
                serde_json::to_writer(&mut stream, &resp)?;
                stream.write_all(b"\n")?;
            }
            Err(err) => {
                eprintln!("Error handling request: {:?}", err);
                // Write malformed response
                stream.write("".as_bytes())?;
                stream.write_all(b"\n")?;
            }
        }
    }

    stream.shutdown(Shutdown::Both)
}

fn handle_request(
    request: Result<String, std::io::Error>,
) -> Result<PrimeResponse, std::io::Error> {
    let request = request?;
    println!("Received request: {}", request);
    let request = serde_json::from_str::<PrimeRequest>(request.as_str())?;

    if request.method != "isPrime" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid method",
        ));
    }

    let is_prime = request.number.fract() == 0.0 && is_prime64(request.number as u64);
    Ok(PrimeResponse {
        method: "isPrime".to_string(),
        prime: is_prime,
    })
}
