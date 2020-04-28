use futures::{channel::mpsc, SinkExt};
use tokio::stream::StreamExt;

use crate::client::Client;
use crate::prelude::*;

pub struct Channel {
	name: String,
	topic: String,
	capacity: usize,
	irc_sender: Sender<Event>,
	broker_sender: Sender<Event>,
	clients: Vec<Client>,
}

impl Channel {
	pub fn new(name: String, capacity: usize, irc_sender: Sender<Event>) -> Channel {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let channel = Channel {
			name,
			topic: String::new(),
			capacity,
			irc_sender,
			broker_sender: sender,
			clients: Vec::new(),
		};

		// tokio::spawn(async move { channel.broker(receiver).await });

		channel
	}

	pub async fn broker(&mut self, mut receiver: Receiver<Event>) {
		while let Some(event) = receiver.next().await {
			match event {
				Event::Connection(stream) => {
					let client = Client::new(stream, self.broker_sender.clone());
					self.clients.push(client);
				}
				Event::Message { nick, msg } => {
					self.send_msg(nick, msg).await;
				}
			}
		}
	}

	pub fn send_event(&self, event: Event) {
		// self.broker_sender.send(event);
	}

	// pub fn is_command(msg: &String) -> bool {
	// 	commands
	// 		.iter()
	// 		.any(|&command| &msg[..commands.len()] == command)
	// }

	pub async fn send_msg(&mut self, nick: String, msg: String) {
		for client in self.clients.iter() {
			if let Some(nick) = client.nick() {
				if nick == nick {
					continue;
				}
				client.send_msg(msg.clone()).await;
			}
		}
	}

	// TODO: Implement a proper error type
	// pub fn insert(&mut self, nick: String, mut writer: WriteHalf<TcpStream>) -> Result<(), ()> {
	// 	if self.capacity == self.clients.len() {
	// 		return Err(());
	// 	}

	// 	if self.clients.contains_key(&nick) {
	// 		return Err(());
	// 	}

	// 	let (sender, mut receiver): (Sender<String>, Receiver<String>) = mpsc::unbounded();
	// 	tokio::spawn(async move {
	// 		while let Some(msg) = receiver.next().await {
	// 			writer.write_all(msg.as_bytes()).await.unwrap();
	// 		}
	// 	});

	// 	self.clients.insert(nick, sender);
	// 	Ok(())
	// }

	// pub fn remove(&mut self, nick: String) -> Option<Sender<String>> {
	// 	self.clients.remove(&nick)
	// }

	// pub fn nicks(&self) -> impl Iterator<Item = &String> + '_ {
	// 	self.clients.keys()
	// }

	// pub fn join(&mut self, nick: String, stream: TcpStream) -> Option<String> {
	// match self.clients.insert(stream) {
	// 	Ok(_) => {
	// 		let res = self
	// 			.clients
	// 			.nicks()
	// 			.map(|s| &**s) // wtf?
	// 			.collect::<Vec<&str>>()
	// 			.join(", ");
	// 		Some(res)
	// 	}
	// 	Err(_) => None,
	// }
	// 	None
	// }
}
