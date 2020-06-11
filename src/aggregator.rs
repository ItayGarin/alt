use tokio::sync::mpsc::{Sender, Receiver};

use crate::events::AltEvent;
use crate::ktrl_client::KtrlIpcReq;
use crate::error::DynError;

pub struct EvAggregator {
    tx: Sender<KtrlIpcReq>,
    rx: Receiver<AltEvent>,
}

impl EvAggregator {
    pub fn new(tx: Sender<KtrlIpcReq>, rx: Receiver<AltEvent>) -> Self {
        Self{tx, rx}
    }

    // async fn send_effect(gateway: &mut ArcEvGateway, effect: &str) -> Result<(), DynError> {
    //     let mut gateway = gateway.lock().await;
    //     let event = format!("IpcDoEffect((fx: {}, val: Press))", effect).to_string();
    //     gateway.tx.send(event).await?;
    //     Ok(())
    // }

    // async fn handle_ivy(gateway: &mut ArcEvGateway, event: ExtEvent) -> Result<(), DynError> {
    //     let effect = match event.state {
    //         ExtEventState::On => "TurnOnLayerAlias(\"ivy\")",
    //         ExtEventState::Off => "TurnOffLayerAlias(\"ivy\")"
    //     };
    //     Self::send_effect(gateway, effect).await
    // }

    pub async fn event_loop(&mut self) -> Result<(), DynError> {
        loop {
            let event = self.rx.recv().await;
            dbg!(event);
        }
    }
}
