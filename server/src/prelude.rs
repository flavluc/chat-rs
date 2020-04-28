use futures::channel::mpsc;
use tokio::net::TcpStream;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

// TODO: split in different enums(i.e. one for `irc` one for `channels` etc.)
pub enum Event {
	Connection(TcpStream),
	Channel { name: String, size: usize },
	Command { nick: String, cmd: String },
	Message { nick: String, msg: String },
}

pub const HALL: &str = "HALL";
pub const MAX_CLIENTS: usize = 2;

pub const KICK: &str = "KICK";
pub const UNICHAN_CMDS: [&str; 1] = [KICK];

pub const JOIN: &str = "JOIN";
pub const MULTICHAN_CMDS: [&str; 1] = [JOIN];

pub enum MsgType {
	Text,
	UniChanCommand,
	MultiChanCommand,
}
