use futures::channel::mpsc;
use tokio::net::TcpStream;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

// TODO: split in different enums(i.e. one for `irc` one for `channels` etc.)
pub enum Event {
	Command(Command),
	Connection(TcpStream),
	Client {
		nick: String,
		sender: Sender<String>,
	},
	Message {
		nick: String,
		msg: String,
	},
}

pub enum MsgType {
	Text,
	Command,
}

pub enum Command {
	Join {
		nick: String,
		channel: String,
		sender: Sender<String>,
	},
}

pub const HALL: &str = "HALL";
pub const MAX_CLIENTS: usize = 2;

pub const KICK: &str = "KICK";
pub const JOIN: &str = "JOIN";
pub const COMMANDS: [&str; 2] = [JOIN, KICK];
