use futures::{channel::mpsc, SinkExt};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

use crate::channel::Channel;
use crate::prelude::*;

pub struct IRC {
	channels: Vec<Sender<Event>>,
	sender: Sender<Event>,
	receiver: Receiver<Event>,
}

impl IRC {
	pub fn start() -> Sender<Event> {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let irc = IRC {
			channels: Vec::new(),
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
		let mut hall = Channel::new(String::from(HALL), MAX_CLIENTS, irc.sender.clone());

		while let Some(event) = irc.receiver.next().await {
			match event {
				Event::Connection(stream) => {
					hall.send(Event::Connection(stream)).await.unwrap();
				}
				Event::Channel { name, size } => {
					irc.insert(name, size);
				}
				Event::Command { nick, cmd } => {
					irc.run_command(nick, cmd);
				}
				_ => {}
			}
		}
	}

	fn insert(&mut self, name: String, size: usize) {
		self
			.channels
			.push(Channel::new(name, size, self.sender.clone()));
	}

	fn run_command(&mut self, nick: String, cmd: String) {
		let op = MULTICHAN_CMDS
			.iter()
			.find(|&command| cmd.len() >= command.len() && &cmd[..command.len()] == *command);

		if let Some(&op) = op {
			match op {
				JOIN => self.join(nick, cmd),
				_ => (),
			};
		}
	}

	fn join(&mut self, nick: String, cmd: String) {}
}
