pub mod thread_pool;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

struct Clients {
	streams: Vec<Option<TcpStream>>,
}

impl Clients {
	fn new(size: usize) -> Clients {
		let mut streams = Vec::new();
		streams.resize_with(size, || None);
		Clients { streams }
	}

	fn push(&mut self, stream: TcpStream) -> Option<usize> {
		for (i, client) in self.streams.iter().enumerate() {
			if client.is_none() {
				self.streams[i] = Some(stream);
				return Some(i);
			}
		}
		None
	}

	fn pop(&mut self, id: usize) {
		self.streams[id] = None;
	}
}

fn handle_client(clients: Arc<Mutex<Clients>>, mut reader: BufReader<TcpStream>, id: usize) {
	let mut data = String::new();
	let size = reader.read_line(&mut data).unwrap();

	let mut clients_guard = clients.lock().unwrap();
	if size > 0 {
		for (i, stream) in clients_guard.streams.iter_mut().enumerate() {
			if i == id {
				continue;
			}

			if let Some(ref mut stream) = stream {
				stream.write_all(data.as_bytes()).unwrap();
			}
		}
		drop(clients_guard);
		tokio::spawn(async move { handle_client(clients, reader, id) });
	} else {
		clients_guard.pop(id);
		println!("client closed connection!");
	}
}

#[tokio::main]
async fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("usage: {} <ip> <port>", args[0]);
		return;
	}
	const MAX_CLIENTS: usize = 2;
	let clients: Arc<Mutex<Clients>> = Arc::new(Mutex::new(Clients::new(MAX_CLIENTS)));

	let addr = format!("{}:{}", args[1], args[2]);
	let listener = TcpListener::bind(&addr).unwrap();

	println!("running server at: {}", &addr);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let mut stream_clone = stream.try_clone().unwrap();
				let clients = Arc::clone(&clients);
				let mut clients_guard = clients.lock().unwrap();
				if let Some(id) = clients_guard.push(stream) {
					drop(clients_guard);
					let reader = BufReader::new(stream_clone);
					tokio::spawn(async move { handle_client(clients, reader, id) });
				} else {
					stream_clone.write_all("server full!".as_bytes()).unwrap();
				}
			}
			Err(e) => {
				println!("connection error: {}", e);
			}
		}
	}
}
