use futures::{channel::mpsc, SinkExt};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

use crate::channel::Channel;
use crate::prelude::*;

pub struct IRC {
	channels: HashMap<String, Sender<Event>>,
	sender: Sender<Event>,
	receiver: Receiver<Event>,
}

impl IRC {
	pub fn start() -> Sender<Event> {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let irc = IRC {
			channels: HashMap::new(),
			sender,
			receiver,
		};

		let sender = irc.sender.clone();
		tokio::spawn(async { IRC::broker(irc).await });
		sender
	}

	pub async fn connection(broker: &mut Sender<Event>, stream: TcpStream) {
		broker.send(Event::Connection(stream)).await.unwrap();
	}

	async fn broker(mut irc: IRC) {
		let mut hall = Channel::new(
			String::from(HALL),
			String::from(ADMIN),
			MAX_CLIENTS,
			irc.sender.clone(),
		);

		while let Some(event) = irc.receiver.next().await {
			match event {
				Event::Connection(stream) => {
					hall.send(Event::Connection(stream)).await.unwrap();
				}
				Event::Command(command) => {
					irc.run_command(command).await;
				}
				_ => {}
			}
		}
	}

	async fn run_command(&mut self, command: Command) {
		match command {
			Command::Join {
				nick,
				channel,
				sender,
			} => {
				self.join(nick, channel, sender).await;
			}
			_ => (),
		};
	}

	async fn join(&mut self, nick: String, channel_name: String, sender: Sender<Action>) {
		let event = Event::Client {
			nick: nick.clone(),
			sender,
		};
		match self.channels.get_mut(&channel_name) {
			Some(channel) => {
				channel.send(event).await.unwrap();
			}
			None => {
				let mut channel =
					Channel::new(channel_name.clone(), nick, MAX_CLIENTS, self.sender.clone());
				channel.send(event).await.unwrap();
				self.channels.insert(channel_name, channel);
			}
		}
	}
}
