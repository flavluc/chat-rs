use futures::{channel::mpsc, SinkExt};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, Lines, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::sync::Mutex;

use crate::prelude::*;

pub struct Client {}

impl Client {
	pub async fn new(stream: TcpStream, channel_sender: Sender<Event>) -> (String, Sender<Action>) {
		let (reader, writer) = tokio::io::split(stream);
		let reader = BufReader::new(reader);
		let mut lines = reader.lines();
		let nick = match lines.next().await {
			None => String::new(),
			Some(line) => line.unwrap(),
		};

		let (sender, receiver): (Sender<Action>, Receiver<Action>) = mpsc::unbounded();

		let channel_sender = Arc::new(Mutex::new(channel_sender));

		tokio::spawn(Self::writer(writer, receiver, Arc::clone(&channel_sender)));
		tokio::spawn(Self::reader(nick.clone(), lines, channel_sender));

		(nick, sender)
	}

	async fn reader(
		nick: String,
		mut lines: Lines<BufReader<ReadHalf<TcpStream>>>,
		channel_sender: Arc<Mutex<Sender<Event>>>,
	) {
		while let Some(msg) = lines.next().await {
			let msg = msg.unwrap().trim().to_string();

			if msg.is_empty() {
				continue;
			}

			let event = Event::Message {
				nick: nick.clone(),
				msg,
			};
			let mut channel_sender = channel_sender.lock().await;
			channel_sender.send(event).await.unwrap();
		}
	}
	async fn writer(
		mut writer: WriteHalf<TcpStream>,
		mut receiver: Receiver<Action>,
		channel_sender: Arc<Mutex<Sender<Event>>>,
	) {
		while let Some(action) = receiver.next().await {
			match action {
				Action::Send(msg) => {
					writer.write_all(msg.as_bytes()).await.unwrap();
				}
				Action::Join(sender) => {
					let mut channel_sender = channel_sender.lock().await;
					*channel_sender = sender;
				}
			}
		}
	}
}
