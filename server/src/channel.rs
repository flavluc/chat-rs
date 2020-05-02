use chrono::prelude::*;
use futures::{channel::mpsc, SinkExt};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json;
use std::collections::HashMap;
use tokio::stream::StreamExt;

use crate::client::Client;
use crate::prelude::*;

struct ChannelInfo {
	name: String,
	topic: String,
	admin: String,
	capacity: usize,
}

pub struct Channel {
	info: ChannelInfo,
	sender: Sender<Event>,
	receiver: Receiver<Event>,
	irc_sender: Sender<Event>,
	clients: HashMap<String, Sender<Action>>,
}

impl Serialize for Channel {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("Channel", 5)?;
		state.serialize_field("name", &self.info.name)?;
		state.serialize_field("topic", &self.info.topic)?;
		state.serialize_field("admin", &self.info.admin)?;
		state.serialize_field("capacity", &self.info.capacity)?;
		let users: Vec<&str> = self.clients.keys().map(|s| s.as_ref()).collect();
		state.serialize_field("users", &users)?;
		state.end()
	}
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

		let data = serde_json::to_string(&self).unwrap();
		let result = ClientResult::CommandSuccess { data };
		sender.send(Action::Send(result)).await.unwrap();

		self.clients.insert(nick, sender);
	}

	fn is_command(msg: &String) -> bool {
		COMMANDS
			.iter()
			.any(|&command| msg.len() >= command.len() && &msg[..command.len()] == command)
	}

	pub async fn send_msg(&mut self, nick: String, message: String) {
		if Channel::is_command(&message) {
			self.run_command(nick, message).await;
		} else {
			let result = ClientResult::Message {
				time: Local::now(),
				nick: nick.clone(),
				message,
			};
			for (client_nick, client) in self.clients.iter_mut() {
				if client_nick != &nick {
					client.send(Action::Send(result.clone())).await.unwrap();
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
			Some(_) => {}
			None => {}
		}
	}

	fn parse_channel_name(mut name: String) -> Result<String, ()> {
		let channel = name.split_off(4).trim().to_string();

		match channel.chars().next().unwrap() {
			'#' | '&' => {
				if channel.contains(" ") {
					return Err(());
				}

				Ok(channel[1..].to_string())
			}
			_ => Err(()),
		}
	}

	async fn join(&mut self, nick: String, msg: String) {
		match Channel::parse_channel_name(msg) {
			Ok(channel) => {
				let client = self.clients.remove(&nick).unwrap();
				let event = Event::Command(Command::Join {
					nick,
					channel,
					sender: client,
				});
				self.irc_sender.send(event).await.unwrap();
			}
			Err(_) => {
				let client = self.clients.get_mut(&nick).unwrap();
				let error = INVALID_CHANNEL_NAME_ERROR.to_string();
				let hint = INVALID_CHANNEL_NAME_HINT.to_string();
				let result = ClientResult::CommandFailure { error, hint };
				client.send(Action::Send(result)).await.unwrap();
			}
		}
	}
}
