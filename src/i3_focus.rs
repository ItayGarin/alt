use tokio_i3ipc::{event::{Event,Subscribe, WindowData, WindowChange}, I3};
use tokio::stream::StreamExt;
use tokio::sync::watch;
use crate::error::DynError;

struct I3FocusListener {
    tx: watch::Sender<String>,
}

impl I3FocusListener {
    async fn new(tx: watch::Sender<String>) -> Self {
        Self{tx}
    }

    async fn handle_window_event(&self, data: Box<WindowData>) {
        if let WindowChange::Focus = data.change {
            let window = match data.container.name {
                Some(name) => name,
                _ => return,
            };

            self.tx.send(window).await;
        }
    }

    async fn event_loop(&self) -> Result<(), DynError> {
        let mut i3 = I3::connect().await?;
        i3.subscribe([Subscribe::Window]).await?;

        let mut listener = self.i3.listen();
        while let Some(event) = listener.next().await {
            match event? {
                Event::Window(ev) => println!("window event {:?}", ev),
                _ => (),
            }
        }

        Ok(())
    }
}
