use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
// use tokio::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum EvState {
    On,
    Off,
}

impl EvState {
    pub fn from_str(string: &str) -> Result<Self, io::Error> {
        match string {
            "on" => Ok(EvState::On),
            "off" => Ok(EvState::Off),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bad event state (should on/off)",
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event<'a> {
    name: &'a str,
    state: EvState,
}

async fn handle_event(event: Event<'_>) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn handle_buf(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let str_buf = std::str::from_utf8(buf)?;
    let split: Vec<&str> = str_buf.split(":").collect();

    if split.len() != 2 {
        let err = Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Bad event (should be <name>:<on/off>)",
        ));
        return Err(err);
    }

    let name = split[0].trim();
    let state = EvState::from_str(split[1].trim())?;
    let event = Event{name, state};

    dbg!(&event);
    handle_event(event).await
}

async fn handle_conn(mut conn: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let is_ok = match conn.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => {
                println!("Client exited");
                return;
            },
            Ok(n) => {
                if let Err(e) = handle_buf(&buf[0..n]).await {
                    eprintln!("client encountered an error; err = {:?}", &e);
                    false
                } else {
                    true
                }
            }
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                false
            }
        };

        let reply: &[u8] = match is_ok {
            true => b"OK\n",
            _ => b"ERROR\n",
        };

        if let Err(e) = conn.write_all(reply).await {
            eprintln!("Failed to send a reply; err = {:?}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ALT: Started");

    let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7332);
    let mut listener = TcpListener::bind(endpoint).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            println!("A new client connected");
            handle_conn(socket).await;
        });
    }
}
