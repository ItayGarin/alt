use anyhow::Result;
use tokio::sync::mpsc;
use log::info;

pub type KtrlIpcReq = String;

pub struct KtrlClient {
    rx: mpsc::Receiver<KtrlIpcReq>,
}

impl KtrlClient {
    pub async fn new(rx: mpsc::Receiver<KtrlIpcReq>) -> Result<Self> {
        Ok(Self { rx })
    }

    pub async fn event_loop(mut self) -> Result<()> {
        let context = tmq::Context::new();
        let mut ktrl_sender = tmq::request(&context).connect("tcp://127.0.0.1:7331")?;
        info!("Connected to ktrl");

        loop {
            let event = match self.rx.recv().await {
                Some(ev) => ev,
                _ => return Ok(()),
            };

            info!("Sending ktrl '{}'", event);

            let msg = tmq::Message::from(&event);
            let multipart = tmq::Multipart::from(msg);
            let ktrl_receiver = ktrl_sender.send(multipart).await?;

            let (mut multi_reply, new_sender) = ktrl_receiver.recv().await?;
            let msg = multi_reply.pop_front().expect("Unexpected reply from ktrl");
            let msg_str = msg.as_str().expect("Failed to convert ktrl's reply to a string");
            info!("KTRL: Replied '{}'", msg_str);

            ktrl_sender = new_sender;
        }
    }
}
