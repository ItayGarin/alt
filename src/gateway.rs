use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;
use log::{info, error};

use anyhow::Result;
use crate::events::*;

pub struct EvGateway {
    tx: Sender<AltEvent>,
}

type ArcEvGateway = Arc<Mutex<EvGateway>>;

impl EvGateway {
    pub async fn new(tx: Sender<AltEvent>) -> Result<Self> {
        Ok(Self { tx })
    }

    async fn init_listen_socket() -> Result<TcpListener> {
        let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7332);
        let listen_socket = TcpListener::bind(endpoint).await?;
        Ok(listen_socket)
    }

    async fn handle_event(gateway: &mut ArcEvGateway, event: ExtEvent) -> Result<()> {
        let mut gateway = gateway.lock().await;
        let event = AltEvent::AltExtEvent(event);
        gateway.tx.send(event).await?;
        Ok(())
    }

    async fn handle_buf(gateway: &mut ArcEvGateway, buf: &[u8]) -> Result<()> {
        let str_buf = std::str::from_utf8(buf)?;
        let split: Vec<&str> = str_buf.split(":").collect();

        anyhow::ensure!(split.len() == 2,
                       "Bad event (should be <name>:<on/off>)");

        let name = split[0].trim().to_string();
        let state = ExtEventState::from_str(split[1].trim())?;
        let event = ExtEvent { name, state };

        Self::handle_event(gateway, event).await
    }

    async fn handle_conn(mut gateway: &mut ArcEvGateway, mut conn: TcpStream) {
        let mut buf: [u8; 1024] = [0; 1024];

        // In a loop, read data from the socket and write the data back.
        loop {
            let is_ok = match conn.read(&mut buf).await {
                // socket closed
                Ok(n) if n == 0 => {
                    info!("Client exited");
                    return;
                }
                Ok(n) => {
                    let res = Self::handle_buf(&mut gateway, &buf[0..n]).await;
                    if let Err(e) = res {
                        error!("client encountered an error; err = {:?}", &e);
                        false
                    } else {
                        true
                    }
                }
                Err(e) => {
                    error!("failed to read from socket; err = {:?}", e);
                    false
                }
            };

            let reply: &[u8] = match is_ok {
                true => b"OK\n",
                _ => b"ERROR\n",
            };

            if let Err(e) = conn.write_all(reply).await {
                error!("Failed to send a reply; err = {:?}", e);
                return;
            }
        }
    }

    pub async fn event_loop(self) -> Result<()> {
        let mut listen_socket = Self::init_listen_socket().await?;
        let arc_gateway = Arc::new(Mutex::new(self));
        loop {
            let (socket, _) = listen_socket.accept().await?;
            let mut arc_clone = arc_gateway.clone();
            tokio::spawn(async move {
                info!("A new client connected");
                Self::handle_conn(&mut arc_clone, socket).await;
            });
        }
    }
}
