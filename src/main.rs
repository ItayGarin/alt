use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
// use tokio::prelude::*;

async fn handle_conn(mut conn: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let _n = match conn.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return,
            Ok(n) => n,
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
    let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7331);
    let mut listener = TcpListener::bind(endpoint).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_conn(socket).await;
        });
    }
}
