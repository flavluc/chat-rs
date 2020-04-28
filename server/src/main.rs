use std::env;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;

use ::chat::irc::IRC;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("usage: {} <ip> <port>", args[0]);
		return;
	}

	let mut broker = IRC::start();

	let addr = format!("{}:{}", args[1], args[2]);
	let mut listener = TcpListener::bind(&addr).await.unwrap();

	println!("running server at: {}", &addr);

	let server = async move {
		let mut incoming = listener.incoming();
		while let Some(stream) = incoming.next().await {
			match stream {
				Ok(stream) => {
					IRC::connection(&mut broker, stream).await;
				}
				Err(e) => {
					println!("connection error: {}", e);
				}
			}
		}
	};

	server.await;
}
