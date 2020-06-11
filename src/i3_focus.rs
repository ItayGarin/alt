use tokio_i3ipc::{event::{Event,Subscribe, WindowData, WindowChange}, I3};
use tokio::stream::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::error::DynError;
use crate::events::*;

pub struct I3FocusListener {
    tx: Sender<AltEvent>,
}

impl I3FocusListener {
    pub fn new(tx: Sender<AltEvent>) -> Self {
        Self{tx}
    }

    async fn handle_window_event(&mut self, data: Box<WindowData>) -> Result<(), DynError> {
        if let WindowChange::Focus = data.change {
            let window = match data.container.name {
                Some(name) => name,
                _ => {
                    println!("empty window focused");
                    return Ok(())
                },
            };

            println!("Focus: {}", window);
            let focus_event = AltEvent::AltFocusEvent(FocusEvent{window});
            self.tx.send(focus_event).await?;
        }

        Ok(())
    }

    pub async fn event_loop(&mut self) -> Result<(), DynError> {
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
}
