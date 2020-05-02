use futures::{channel::mpsc, SinkExt};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use tokio::stream::StreamExt;

use crate::client::Client;
use crate::prelude::*;

#[derive(Serialize)]
struct ChannelInfo {
	name: String,
	topic: String,
	admin: String,
	capacity: usize,
}

pub struct Channel {
	info: ChannelInfo,
	irc_sender: Sender<Event>,
	sender: Sender<Event>,
	receiver: Receiver<Event>,
	clients: HashMap<String, Sender<Action>>,
}

impl Channel {
	pub fn new(
		name: String,
		admin: String,
		capacity: usize,
		irc_sender: Sender<Event>,
	) -> Sender<Event> {
		let info = ChannelInfo {
			name,
			capacity,
			admin,
			topic: String::from(DEFAULT_TOPIC),
		};
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let channel = Channel {
			info,
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
		let join_action = Action::Join(self.sender.clone());
		sender.send(join_action).await.unwrap();

		let mut users: Vec<&str> = self.clients.keys().map(|s| s.as_ref()).collect();
		users.push(&nick);

		let serialized_info = serde_json::to_string(&self.info).unwrap();
		let welcome_action = Action::Send(serialized_info);
		sender.send(welcome_action).await.unwrap();

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
			for (client_nick, client) in self.clients.iter_mut() {
				if client_nick != &nick {
					client.send(Action::Send(msg.clone())).await.unwrap();
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
