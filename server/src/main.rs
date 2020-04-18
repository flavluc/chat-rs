pub mod thread_pool;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

fn handle_client(streams: Arc<Mutex<Vec<TcpStream>>>, mut reader: BufReader<TcpStream>, id: usize) {
	let mut data = String::new();
	reader.read_line(&mut data).unwrap();

	let clients = streams.lock().unwrap();
	for (i, mut stream) in clients.iter().enumerate() {
		if i != id {
			stream.write_all(data.as_bytes()).unwrap();
		}
	}
	drop(clients);
	tokio::spawn(async move { handle_client(streams, reader, id) });
}

#[tokio::main]
async fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("usage: {} <ip> <port>", args[0]);
		return;
	}

	let streams: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

	let addr = format!("{}:{}", args[1], args[2]);
	let listener = TcpListener::bind(&addr).unwrap();
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let reader = BufReader::new(stream.try_clone().unwrap());
				let streams = Arc::clone(&streams);
				let mut clients = streams.lock().unwrap();
				let len = clients.len();
				clients.push(stream);
				drop(clients);
				tokio::spawn(async move { handle_client(streams, reader, len) });
			}
			Err(e) => {
				println!("Connection error: {}", e);
			}
		}
	}

	println!("running server at: {}", &addr);
}
