mod error;
mod ktrl_client;
mod gateway;
mod i3_focus;
mod events;

use error::DynError;
use ktrl_client::KtrlClient;
use gateway::EvGateway;
use i3_focus::I3FocusListener;

use tokio::sync::mpsc;
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");

    let (focus_tx, focus_rx) = watch::channel("".to_string());
    let i3listener = I3FocusListener::new(focus_tx);

    let (gateway_tx, gateway_rx) = mpsc::channel(1);
    let gateway = EvGateway::new(gateway_tx).await?;
    let client = KtrlClient::new(gateway_rx).await?;

    let (gateway_result, client_res, i3_res) =
        tokio::join!(
            gateway.event_loop(),
            client.event_loop(),
            i3listener.event_loop(),
        );

    gateway_result?;
    client_res?;
    i3_res?;

    Ok(())
}
