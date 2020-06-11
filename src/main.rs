mod error;
mod ktrl_client;
mod server;
mod i3_focus;

use error::DynError;
use ktrl_client::KtrlClient;
use server::AltServer;
use i3_focus::I3FocusListener;

use tokio::sync::mpsc;
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");

    let (focus_tx, focus_rx) = watch::channel("".to_string());
    let i3listener = I3FocusListener::new(focus_tx);

    let (server_tx, server_rx) = mpsc::channel(1);
    let server = AltServer::new(server_tx).await?;
    let client = KtrlClient::new(server_rx).await?;

    let (server_result, client_res, i3_res) =
        tokio::join!(
            server.event_loop(),
            client.event_loop(),
            i3listener.event_loop(),
        );

    server_result?;
    client_res?;
    i3_res?;

    Ok(())
}
