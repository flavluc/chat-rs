pub mod thread_pool;

use futures::{channel::mpsc, SinkExt};
use std::env;
use tokio::io::{AsyncBufReadExt, BufReader, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;

use ::chat::channels::Channel;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

enum Event {
	Connection {
		nick: String,
		stream: WriteHalf<TcpStream>,
	},
	Message {
		nick: String,
		msg: String,
	},
}

async fn handler(stream: TcpStream, mut sender: Sender<Event>) -> Result<()> {
	let (reader, writer) = tokio::io::split(stream);
	let reader = BufReader::new(reader);
	let mut lines = reader.lines();
	let nick = match lines.next().await {
		// TODO: Implement a proper error type
		None => Err("client disconnected")?,
		Some(line) => line?,
	};

	sender
		.send(Event::Connection {
			nick: nick.clone(),
			stream: writer,
		})
		.await
		.unwrap();

	while let Some(msg) = lines.next().await {
		sender
			.send(Event::Message {
				nick: nick.clone(),
				msg: msg.unwrap(),
			})
			.await
			.unwrap();
	}
	Ok(())
}

async fn broker(mut channel: Channel, mut receiver: Receiver<Event>) {
	while let Some(event) = receiver.next().await {
		match event {
			Event::Message { nick, msg } => {
				channel.send(nick, msg).await;
			}
			Event::Connection { nick, stream } => {
				match channel.insert(nick, stream) {
					Ok(_) => {}
					Err(_) => {}
				};
			}
		}
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
	const HALL: &str = "HALL";
	let channel: Channel = Channel::new(String::from(HALL), MAX_CLIENTS);

	let addr = format!("{}:{}", args[1], args[2]);
	let mut listener = TcpListener::bind(&addr).await.unwrap();

	println!("running server at: {}", &addr);

	let (sender, receiver) = mpsc::unbounded();

	tokio::spawn(async move { broker(channel, receiver).await });

	let server = async move {
		let mut incoming = listener.incoming();
		while let Some(stream) = incoming.next().await {
			match stream {
				Ok(stream) => {
					let sender = sender.clone();
					tokio::spawn(handler(stream, sender));
				}
				Err(e) => {
					println!("connection error: {}", e);
				}
			}
		}
	};

	server.await;
}
