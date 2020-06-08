mod server;
use server::AltServer;
use server::DynError;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("ALT: Started");
    let (tx, rx) = mpsc::channel(32);
    let alt = AltServer::new(tx).await?;
    alt.event_loop().await?;
    Ok(())
}
