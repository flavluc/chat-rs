pub mod thread_pool;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use thread_pool::ThreadPool;

fn handle_client(mut stream: TcpStream) {
	let mut reader = BufReader::new(stream.try_clone().unwrap());
	loop {
		let mut data = String::new();
		reader.read_line(&mut data).unwrap();

		print!("{}", data);

		stream.write_all(data.as_bytes()).unwrap();
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("usage: {} <ip> <port>", args[0]);
		return;
	}

	let pool = ThreadPool::new(2);

	let addr = format!("{}:{}", args[1], args[2]);

	let listener = TcpListener::bind(addr).unwrap();
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				pool.execute(move || handle_client(stream));
			}
			Err(e) => {
				println!("Connection error: {}", e);
			}
		}
	}
}
