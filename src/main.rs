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

use tmq::request_reply::RequestSender;
use tmq::{request, Context};
// use futures::SinkExt;

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

struct Alt {
    context: Context,
}

type ArcAlt = Arc<Mutex<Alt>>;
type DynError = Box<dyn std::error::Error>;

impl Alt {
    pub async fn new() -> Result<Self, DynError> {
        let context = Context::new();

        Ok(Alt {
            context,
        })
    }

    pub async fn init_listen_socket() -> Result<TcpListener, DynError> {
        let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7332);
        let listen_socket = TcpListener::bind(endpoint).await?;
        Ok(listen_socket)
    }

    async fn handle_ivy(ktrl_socket: RequestSender, event: Event<'_>) -> Result<RequestSender, DynError> {
        let msg = tmq::Message::from("IpcDoEffect((fx: NoOp, val: Press))");
        let multipart = tmq::Multipart::from(msg);
        let reciver = ktrl_socket.send(multipart).await?;
        Ok(())
    }

    async fn handle_event(ktrl_socket: RequestSender, event: Event<'_>) -> Result<RequestSender, DynError> {
        match event.name {
            "ivy" => {
                Self::handle_ivy(ktrl_socket, event).await?;
                ()
            }
            _ => (),
        }

        Ok(())
    }

    async fn handle_buf(ktrl_socket: RequestSender, buf: &[u8]) -> Result<RequestSender, DynError> {
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
        Self::handle_event(ktrl_socket, event).await
    }

    async fn handle_conn(alt: &ArcAlt, mut conn: TcpStream) {
        let mut buf: [u8; 1024] = [0; 1024];

        let mut ktrl_socket = {
            let alt = alt.lock().await;
            match request(&alt.context).connect("tcp://127.0.0.1:7331") {
                Ok(socket) => socket,
                Err(e) => {
                    println!("Failed to connect to ktrl");
                    return;
                }
            }
        };

        // In a loop, read data from the socket and write the data back.
        loop {
            let is_ok = match conn.read(&mut buf).await {
                // socket closed
                Ok(n) if n == 0 => {
                    println!("Client exited");
                    return;
                }
                Ok(n) => {
                    let res = Self::handle_buf(ktrl_socket, &buf[0..n]).await;
                    if let Err(e) = res {
                        eprintln!("client encountered an error; err = {:?}", &e);
                        false
                    } else {
                        ktrl_socket = res.unwrap();
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

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");
    let alt = Alt::new().await?;
    alt.event_loop().await?;
    Ok(())
}
