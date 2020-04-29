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
	clients: HashMap<String, Sender<Action>>,
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
					let mut channel_sender = channel.sender.clone();
					tokio::spawn(async move {
						let (nick, sender) = Client::new(stream, channel_sender.clone()).await;
						channel_sender
							.send(Event::Client { nick, sender })
							.await
							.unwrap();
					});
				}
				Event::Message { nick, msg } => {
					channel.send_msg(nick, msg).await;
				}
				Event::Client { nick, sender } => {
					channel.client(nick, sender).await;
				}
				_ => {}
			}
		}
	}

	async fn client(&mut self, nick: String, mut sender: Sender<Action>) {
		sender
			.send(Action::Join(self.sender.clone()))
			.await
			.unwrap();
		self.clients.insert(nick, sender);
	}

	fn is_command(msg: &String) -> bool {
		COMMANDS
			.iter()
			.any(|&command| msg.len() >= command.len() && &msg[..command.len()] == command)
	}

	pub async fn send_msg(&mut self, nick: String, msg: String) {
		if Channel::is_command(&msg) {
			self.run_command(nick, msg).await;
		} else {
			for (nick_to_send, sender) in self.clients.iter_mut() {
				if nick_to_send != &nick {
					sender.send(Action::Send(msg.clone())).await.unwrap();
				}
			}
		}
	}

	async fn run_command(&mut self, nick: String, cmd: String) {
		let op = COMMANDS
			.iter()
			.find(|&command| &cmd[..command.len()] == *command);

		if let Some(&op) = op {
			match op {
				JOIN => {
					self.join(nick, cmd).await;
				}
				KICK => {
					self.kick(nick, cmd).await;
				}
				_ => (),
			};
		}
	}

	async fn kick(&mut self, nick: String, mut msg: String) {
		let kicked_nick = msg.split_off(4);
		match self.clients.get_mut(kicked_nick.trim()) {
			Some(kicked_sender) => {
				kicked_sender
					.send(Action::Send(String::from("KICKED!")))
					.await
					.unwrap();
			}
			None => {
				let kicker_sender = self.clients.get_mut(&nick).unwrap();
				kicker_sender
					.send(Action::Send(String::from("USER DOES NOT EXIST!")))
					.await
					.unwrap();
			}
		}
	}

	async fn join(&mut self, nick: String, mut msg: String) {
		let sender = self.clients.remove(&nick).unwrap();
		let channel = msg.split_off(4).trim().to_string();

		let event = Event::Command(Command::Join {
			nick,
			channel,
			sender,
		});
		self.irc_sender.send(event).await.unwrap();
	}
}
