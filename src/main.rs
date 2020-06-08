mod error;
mod ktrl_client;
mod server;

use error::DynError;
use ktrl_client::KtrlClient;
use server::AltServer;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");
    let (tx, rx) = mpsc::channel(1);
    let server = AltServer::new(tx).await?;
    let client = KtrlClient::new(rx).await?;

    let (server_result, client_res) = tokio::join!(server.event_loop(), client.event_loop());
    server_result?;
    client_res?;

    Ok(())
}
