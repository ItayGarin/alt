use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
// use tokio::prelude::*;

pub enum EvState {
    On,
    Off
}

impl EvState {
    pub fn from_str(string: &str) -> Result<Self, io::Error> {
        match string {
            "on" => Ok(EvState::On),
            "off" => Ok(EvState::Off),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Bad event state (should on/off)")),
        }
    }
}

async fn handle_buf(buf: &[u8]) -> Result<(), io::Error> {
    let str_buf = std::str::from_utf8(buf)
        .expect("Failed to convert a packet into a string");

    let split: Vec<&str> =
        str_buf.split(":")
        .collect();

    if split.len() != 2 {
        let err = io::Error::new(io::ErrorKind::InvalidInput, "Bad event (should be <name>:<on/off>)");
        return Err(err);
    }

    let event = split[0].trim().to_owned();
    let state = split[1].trim().to_owned();

    dbg!((event, state));
    Ok(())
}

async fn handle_conn(mut conn: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        match conn.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return,
            Ok(n) => handle_buf(&buf[0..n]).await,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return
            }
        };

        // // Write the data back
        // if let Err(e) = conn.write_all(&buf[0..n]).await {
        //     eprintln!("failed to write to socket; err = {:?}", e);
        //     return
        // }
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
