use std::env;
use std::io::{stdin, BufRead, BufReader, Error, Write};
use std::net::TcpStream;
use std::thread;

fn msg_receiver(stream: TcpStream) -> Result<(), Error> {
	let mut reader = BufReader::new(stream);
	loop {
		let mut data = String::new();
		reader.read_line(&mut data)?;
		print!("{}", data);
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("usage: {} <ip> <port>", args[0]);
		return;
	}

	let addr = format!("{}:{}", args[1], args[2]);

	match TcpStream::connect(addr) {
		Ok(mut stream) => {
			let reader = stream.try_clone().unwrap();
			thread::spawn(move || msg_receiver(reader));

			loop {
				let mut data = String::new();
				stdin().read_line(&mut data).unwrap();
				stream.write_all(data.as_bytes()).unwrap();
			}
		}
		Err(e) => {
			println!("Connection error: {}", e);
		}
	}
}
