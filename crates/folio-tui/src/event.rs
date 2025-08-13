use crossterm::event::{Event, EventStream};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        tokio::spawn({
            let tx = tx.clone();
            async move {
                let mut reader = EventStream::new();
                while let Some(event) = reader.next().await {
                    match event {
                        Ok(Event::Key(key_event)) => {
                            if tx.send(AppEvent::Key(key_event)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tick_rate).await;
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx, _tx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
