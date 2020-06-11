use tokio_i3ipc::{event::{Event,Subscribe, WindowData, WindowChange}, I3};
use tokio::stream::StreamExt;
use tokio::sync::watch;
use crate::error::DynError;

pub struct I3FocusListener {
    tx: watch::Sender<String>,
}

impl I3FocusListener {
    pub fn new(tx: watch::Sender<String>) -> Self {
        Self{tx}
    }

    async fn handle_window_event(&self, data: Box<WindowData>) -> Result<(), DynError> {
        if let WindowChange::Focus = data.change {
            let window = match data.container.name {
                Some(name) => name,
                _ => {
                    println!("empty window focused");
                    return Ok(())
                },
            };

            println!("Focus: {}", window);
            self.tx.broadcast(window)?;
        }

        Ok(())
    }

    pub async fn event_loop(&self) -> Result<(), DynError> {
        let mut i3 = I3::connect().await?;
        i3.subscribe([Subscribe::Window]).await?;

        let mut listener = i3.listen();
        while let Some(event) = listener.next().await {
            match event? {
                Event::Window(ev) => self.handle_window_event(ev).await?,
                _ => (),
            }
        }

        Ok(())
    }

    pub async fn get_focused(rx: &mut watch::Receiver<String>) -> String {
        rx.recv().await
            .expect("i3FocusListener unexpectedly quit")
    }
}
