use futures::{channel::mpsc, SinkExt};
use std::collections::HashMap;
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
	clients: HashMap<String, Sender<String>>,
}

impl Channel {
	pub fn new(name: String, capacity: usize, irc_sender: Sender<Event>) -> Sender<Event> {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let channel = Channel {
			name,
			capacity,
			topic: String::new(),
			clients: HashMap::new(),
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
					let (nick, sender) = Client::new(stream, channel.sender.clone()).await;
					channel.clients.insert(nick, sender);
				}
				Event::Message { nick, msg } => {
					channel.send_msg(nick, msg).await;
				}
				_ => {}
			}
		}
	}

	fn msg_type(msg: &String) -> MsgType {
		let is_multi_command = MULTICHAN_CMDS
			.iter()
			.any(|&command| msg.len() >= command.len() && &msg[..command.len()] == command);

		if is_multi_command {
			return MsgType::MultiChanCommand;
		}

		let is_single_command = UNICHAN_CMDS
			.iter()
			.any(|&command| msg.len() >= command.len() && &msg[..command.len()] == command);

		if is_single_command {
			return MsgType::UniChanCommand;
		}

		MsgType::Text
	}

	pub async fn send_msg(&mut self, nick: String, msg: String) {
		match Channel::msg_type(&msg) {
			MsgType::MultiChanCommand => {
				let event = Event::Command { nick, cmd: msg };
				self.irc_sender.send(event).await.unwrap();
			}
			MsgType::UniChanCommand => {
				self.run_command(nick, msg).await;
			}
			MsgType::Text => {
				for (nick_to_send, sender) in self.clients.iter_mut() {
					if nick_to_send != &nick {
						sender.send(msg.clone()).await.unwrap();
					}
				}
			}
		}
	}

	async fn run_command(&mut self, nick: String, cmd: String) {
		let op = UNICHAN_CMDS
			.iter()
			.find(|&command| &cmd[..command.len()] == *command);

		if let Some(&op) = op {
			match op {
				KICK => {
					self.kick(nick, cmd).await;
				}
				_ => (),
			};
		}
	}

	async fn kick(&mut self, nick: String, mut cmd: String) {
		let kicked_nick = cmd.split_off(4);
		match self.clients.get_mut(kicked_nick.trim()) {
			Some(kicked_sender) => {
				kicked_sender.send(String::from("KICKED!")).await.unwrap();
			}
			None => {
				let kicker_sender = self.clients.get_mut(&nick).unwrap();
				kicker_sender
					.send(String::from("USER DOES NOT EXIST!"))
					.await
					.unwrap();
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
