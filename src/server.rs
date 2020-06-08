use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
// use tokio::prelude::*;

use std::sync::Arc;
use tokio::sync::mpsc;

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

pub struct AltServer {
    tx: mpsc::Sender<String>,
}

type ArcAltServer = Arc<Mutex<AltServer>>;
pub type DynError = Box<dyn std::error::Error>;

impl AltServer {
    pub async fn new(tx: mpsc::Sender<String>) -> Result<Self, DynError> {
        Ok(AltServer {
            tx,
        })
    }

    pub async fn init_listen_socket() -> Result<TcpListener, DynError> {
        let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7332);
        let listen_socket = TcpListener::bind(endpoint).await?;
        Ok(listen_socket)
    }

    async fn handle_ivy(server: &ArcAltServer, event: Event<'_>) -> Result<(), DynError> {
        Ok(())
    }

    async fn handle_event(server: &ArcAltServer, event: Event<'_>) -> Result<(), DynError> {
        match event.name {
            "ivy" => {
                Self::handle_ivy(server, event).await?;
                ()
            }
            _ => (),
        }

        Ok(())
    }

    async fn handle_buf(server: &ArcAltServer, buf: &[u8]) -> Result<(), DynError> {
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
        let event = Event { name, state };

        dbg!(&event);
        Self::handle_event(server, event).await
    }

    async fn handle_conn(server: &ArcAltServer, mut conn: TcpStream) {
        let mut buf: [u8; 1024] = [0; 1024];

        // In a loop, read data from the socket and write the data back.
        loop {
            let is_ok = match conn.read(&mut buf).await {
                // socket closed
                Ok(n) if n == 0 => {
                    println!("Client exited");
                    return;
                }
                Ok(n) => {
                    let res = Self::handle_buf(&server, &buf[0..n]).await;
                    if let Err(e) = res {
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

    pub async fn event_loop(self) -> Result<(), DynError> {
        let mut listen_socket = Self::init_listen_socket().await?;
        let arc_alt = Arc::new(Mutex::new(self));
        loop {
            let (socket, _) = listen_socket.accept().await?;
            let arc_clone = arc_alt.clone();
            tokio::spawn(async move {
                println!("A new client connected");
                Self::handle_conn(&arc_clone, socket).await;
            });
        }
    }
}
