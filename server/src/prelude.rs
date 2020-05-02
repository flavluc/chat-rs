use chrono::prelude::*;
use futures::channel::mpsc;
use serde::Serialize;
use tokio::net::TcpStream;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

// TODO: split in different enums(i.e. one for `irc` one for `channels` etc.)
pub enum Event {
	Command(Command),
	Connection(TcpStream),
	Client {
		nick: String,
		sender: Sender<Action>,
	},
	Message {
		nick: String,
		msg: String,
	},
}

pub enum Command {
	Join {
		nick: String,
		channel: String,
		sender: Sender<Action>,
	},
}

// TODO: choose a better name
pub enum Action {
	Send(ClientResult),
	Join(Sender<Event>),
}

#[derive(Serialize, Clone)]
pub enum ClientResult {
	Message {
		time: DateTime<Local>,
		nick: String,
		message: String,
	},
	CommandSuccess {
		data: String,
	},
	CommandFailure {
		error: String,
		hint: String,
	},
}

pub const HALL: &str = "HALL";
pub const HALL_ADMIN: &str = "SERVER";
pub const DEFAULT_TOPIC: &str = "Default Topic of the Channel.";
pub const MAX_CLIENTS: usize = 5;

pub const KICK: &str = "KICK";
pub const JOIN: &str = "JOIN";
pub const COMMANDS: [&str; 2] = [JOIN, KICK];

pub const INVALID_CHANNEL_NAME_ERROR: &str = "INVALID_CHANNEL_NAME_ERROR";
pub const INVALID_CHANNEL_NAME_HINT: &str =
	"Channels names are strings (beginning with a '&' or '#' character) of
length up to 200 characters.  Apart from the the requirement that the
first character being either '&' or '#'; the only restriction on a
channel name is that it may not contain any spaces (' '), a control G
(^G or ASCII 7), or a comma (',' which is used as a list item
separator by the protocol).";
