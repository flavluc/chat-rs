use futures::{channel::mpsc, SinkExt};
use std::collections::HashMap;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::stream::StreamExt;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub struct Channel {
	name: String,
	topic: String,
	capacity: usize,
	clients: HashMap<String, Sender<String>>,
}

impl Channel {
	pub fn new(name: String, capacity: usize) -> Channel {
		Channel {
			name,
			topic: String::new(),
			capacity,
			clients: HashMap::new(),
		}
	}

	// TODO: Implement a proper error type
	pub fn insert(&mut self, nick: String, mut writer: WriteHalf<TcpStream>) -> Result<(), ()> {
		if self.capacity == self.clients.len() {
			return Err(());
		}

		if self.clients.contains_key(&nick) {
			return Err(());
		}

		let (sender, mut receiver): (Sender<String>, Receiver<String>) = mpsc::unbounded();
		tokio::spawn(async move {
			while let Some(msg) = receiver.next().await {
				writer.write(msg.as_bytes()).await.unwrap();
			}
		});

		self.clients.insert(nick, sender);
		Ok(())
	}

	pub async fn send(&mut self, nick: String, msg: String) {
		match self.clients.get(&nick) {
			Some(_) => {
				for (client_nick, mut sender) in self.clients.iter() {
					if client_nick != &nick {
						sender.send(msg.clone()).await.unwrap();
					}
				}
			}
			None => {}
		}
	}

	pub fn remove(&mut self, nick: String) -> Option<Sender<String>> {
		self.clients.remove(&nick)
	}

	pub fn nicks(&self) -> impl Iterator<Item = &String> + '_ {
		self.clients.keys()
	}

	// fn join(&mut self, nick: String, stream: TcpStream) -> Option<String> {
	// 	match self.clients.insert(stream) {
	// 		Ok(_) => {
	// 			let res = self
	// 				.clients
	// 				.nicks()
	// 				.map(|s| &**s) // wtf?
	// 				.collect::<Vec<&str>>()
	// 				.join(", ");
	// 			Some(res)
	// 		}
	// 		Err(_) => None,
	// 	}
	// }
}

pub struct Channels {
	channels: Vec<Channel>,
}

impl Channels {
	pub fn new() -> Channels {
		Channels {
			channels: Vec::new(),
		}
	}

	pub fn insert(&mut self, name: String, size: usize) {
		self.channels.push(Channel::new(name, size));
	}

	pub fn join(&mut self, name: String, nick: String, stream: TcpStream) -> Option<String> {
		// 	match self
		// 		.channels
		// 		.iter_mut()
		// 		.find(|channel| channel.name == name)
		// 	{
		// 		Some(channel) => channel.join(nick, stream),
		// 		None => None,
		// 	}
		None
	}
}
