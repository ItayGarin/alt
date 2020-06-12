mod error;
mod events;

mod gateway;
mod i3_focus;
mod aggregator;
mod ktrl_client;

use error::DynError;
use gateway::EvGateway;
use i3_focus::I3FocusListener;
use aggregator::EvAggregator;
use ktrl_client::KtrlClient;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");

    let (agg_tx, agg_rx) = mpsc::channel(1);
    let (ktrl_tx, ktrl_rx) = mpsc::channel(1);

    let gateway = EvGateway::new(agg_tx.clone()).await?;
    let mut i3listener = I3FocusListener::new(agg_tx);
    let mut aggregator = EvAggregator::new(ktrl_tx, agg_rx);
    let client = KtrlClient::new(ktrl_rx).await?;

    let (gateway_result, i3_res, agg_res, client_res) =
        tokio::join!(
            gateway.event_loop(),
            i3listener.event_loop(),
            aggregator.event_loop(),
            client.event_loop(),
        );

    gateway_result?;
    i3_res?;
    agg_res?;
    client_res?;

    Ok(())
}
