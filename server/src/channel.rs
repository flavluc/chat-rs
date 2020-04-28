use futures::{channel::mpsc, SinkExt};
use tokio::stream::StreamExt;

use crate::client::Client;
use crate::prelude::*;

pub struct Channel {
	name: String,
	topic: String,
	capacity: usize,
	irc_sender: Sender<Event>,
	sender: Sender<Event>,
	receiver: Receiver<Event>,
	clients: Vec<Client>,
}

impl Channel {
	pub fn new(name: String, capacity: usize, irc_sender: Sender<Event>) -> Sender<Event> {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let channel = Channel {
			name,
			capacity,
			topic: String::new(),
			clients: Vec::new(),
			irc_sender,
			sender,
			receiver,
		};

		let sender = channel.sender.clone();
		tokio::spawn(async move { Channel::broker(channel).await });

		sender
	}

	pub async fn broker(mut channel: Channel) {
		while let Some(event) = channel.receiver.next().await {
			match event {
				Event::Connection(stream) => {
					let client = Client::new(stream, channel.sender.clone()).await;
					channel.clients.push(client);
				}
				Event::Message { nick, msg } => {
					channel.send_msg(nick, msg).await;
				}
				_ => {}
			}
		}
	}

	// pub fn is_command(msg: &String) -> bool {
	// 	commands
	// 		.iter()
	// 		.any(|&command| &msg[..commands.len()] == command)
	// }

	pub async fn send_msg(&mut self, nick: String, msg: String) {
		for client in self.clients.iter_mut() {
			if client.nick != nick {
				client.sender.send(msg.clone()).await.unwrap();
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
