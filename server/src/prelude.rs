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
	Send(String),
	Join(Sender<Event>),
}

pub const HALL: &str = "HALL";
pub const MAX_CLIENTS: usize = 5;

pub const KICK: &str = "KICK";
pub const JOIN: &str = "JOIN";
pub const COMMANDS: [&str; 2] = [JOIN, KICK];
