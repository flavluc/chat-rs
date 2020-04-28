use futures::{channel::mpsc, SinkExt};
use tokio::io::{AsyncBufReadExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::stream::StreamExt;

use crate::prelude::*;

pub struct Client {
	nick: Option<String>,
	client_sender: Option<Sender<String>>,
	channel_sender: Sender<Event>,
}

impl Client {
	pub fn new(stream: TcpStream, channel_sender: Sender<Event>) -> Client {
		let (sender, receiver): (Sender<Event>, Receiver<Event>) = mpsc::unbounded();
		let client = Client {
			nick: None,
			client_sender: None,
			channel_sender,
		};

		// tokio::spawn(async move { client.handler(stream).await });

		client
	}
	async fn handler(&mut self, stream: TcpStream) {
		let (reader, writer) = tokio::io::split(stream);
		let reader = BufReader::new(reader);
		let mut lines = reader.lines();
		let nick = match lines.next().await {
			None => String::new(),
			Some(line) => line.unwrap(),
		};

		let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::unbounded();

		self.nick = Some(nick);
		self.client_sender = Some(sender);

		let channel_sender = self.channel_sender.clone();
		let nick = self.nick.clone().unwrap();

		// tokio::spawn(async move { Self::reader(reader, channel_sender, nick).await });
		tokio::spawn(async move { Self::writer(writer, receiver).await });
	}
	async fn reader(
		reader: BufReader<ReadHalf<TcpStream>>,
		mut channel_sender: Sender<Event>,
		nick: String,
	) {
		let mut lines = reader.lines();
		while let Some(msg) = lines.next().await {
			let event = Event::Message {
				nick: nick.clone(),
				msg: msg.unwrap(),
			};
			channel_sender.send(event).await.unwrap();
		}
	}
	async fn writer(mut writer: WriteHalf<TcpStream>, mut receiver: Receiver<String>) {
		while let Some(msg) = receiver.next().await {
			writer.write_all(msg.as_bytes()).await.unwrap();
		}
	}

	pub async fn send_msg(&self, msg: String) {
		// if let Some(mut sender) = &self.client_sender {
		// 	sender.send(msg);
		// }
	}

	pub fn nick(&self) -> Option<String> {
		// self.nick
		None
	}
}
