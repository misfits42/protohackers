use std::io::{prelude::*, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

use num_bigint::{BigInt, ToBigInt};
use serde_json::{map::Entry, Value};

use protohackers::utils::thread::ThreadPool;

const PROBLEM_NAME: &str = "Prime Time";
const PROBLEM_NUMBER: u64 = 1;

const IP_ADDR: &str = "0.0.0.0";
const PORT_TCP: u16 = 80;
const NUM_WORKERS: usize = 10;

/// Entry point function for Protohackers problem 001.
pub fn main() {
    println!(
        "Protohackers // Problem {} - \"{}\"",
        PROBLEM_NUMBER, PROBLEM_NAME
    );
    println!("==================================================");
    // Bind TCP listener to accept incoming connections
    let addr = format!("{}:{}", IP_ADDR, PORT_TCP);
    let listener = TcpListener::bind(addr).unwrap();
    println!("[+] Listening on: {}:{} ...", IP_ADDR, PORT_TCP);
    // Create thread pool and handle incoming connections
    let threadpool = ThreadPool::new(NUM_WORKERS);
    for stream in listener.incoming().flatten() {
        println!(
            "[+] Incoming connection from: {}",
            stream.peer_addr().unwrap()
        );
        threadpool.execute(|| handle_connection(stream));
    }
}

/// Handle connection from client // Processes JSON requests from the clients and responds based
/// on the method requested.
fn handle_connection(stream: TcpStream) {
    // Create the bufreader
    let mut buf_reader = {
        if let Ok(rstream) = stream.try_clone() {
            BufReader::new(rstream)
        } else {
            return;
        }
    };
    // Create the bufwriter
    let mut buf_writer = {
        if let Ok(wstream) = stream.try_clone() {
            BufWriter::new(wstream)
        } else {
            return;
        }
    };
    // Keep processing requests from client until stream closed or malformed request received
    loop {
        // Read line from stream
        let mut buf = String::new();
        match buf_reader.read_line(&mut buf) {
            Ok(size) => {
                if size == 0 {
                    return;
                }
            },
            Err(_) => return,
        }
        // Process data
        if let Some(n) = extract_number(&buf) {
            let result = is_prime(&n);
            send_conforming_response(&mut buf_writer, result);
        } else {
            send_malformed_response(&mut buf_writer);
            return;
        }
    }
}

/// Extracts the number from the request sent from the client. Returns None if the request is
/// malformed JSON, request fields are missing or required fields contain invalid values.
fn extract_number(buf: &str) -> Option<BigInt> {
    if let Ok(v) = serde_json::from_str::<Value>(buf) {
        match v {
            Value::Object(mut map) => {
                // Check that request contains "method" field with valid value
                if let Entry::Occupied(e) = map.entry("method") {
                    let val = e.get();
                    if !val.is_string() || (val.is_string() && val.as_str().unwrap() != "isPrime") {
                        return None;
                    }
                } else {
                    return None;
                }
                // Check that request contains "number" field with valid value
                if let Entry::Occupied(e) = map.entry("number") {
                    let val = e.get().to_string();
                    // Replace any floats with a non-prime number
                    if val.contains('.') {
                        return Some(0.to_bigint().unwrap());
                    }
                    if let Ok(n) = BigInt::from_str(&val) {
                        return Some(n);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
            _ => {
                return None;
            }
        }
    }
    None
}

/// Checks if "n" is a prime number.
fn is_prime(n: &BigInt) -> bool {
    // let mut n = n.clone();
    if *n <= 1.to_bigint().unwrap() {
        return false;
    } else if *n <= 3.to_bigint().unwrap() {
        return true;
    }
    let mut i = 2.to_bigint().unwrap();
    let upper = &(n.sqrt() + 1.to_bigint().unwrap());
    while i <= *upper {
        let n1 = n.clone();
        let i1 = i.clone();
        if n1 % i1 == 0.to_bigint().unwrap() {
            return false;
        }
        i += 1.to_bigint().unwrap();
    }
    true
}

/// Sends a malformed response to the client.
fn send_malformed_response(buf_writer: &mut BufWriter<TcpStream>) {
    if buf_writer.write("{}".as_bytes()).is_ok() {}
    if buf_writer.flush().is_ok() {}
}

/// Sends a conforming response to the client.
fn send_conforming_response(buf_writer: &mut BufWriter<TcpStream>, result: bool) {
    let response = format!("{{\"method\":\"isPrime\",\"prime\":{result}}}\n");
    if buf_writer.write(response.as_bytes()).is_ok() {}
    if buf_writer.flush().is_ok() {}
}
