use crossterm::event::{Event, KeyEvent};
use tokio::sync::mpsc;

use omc_core::types::{Channel, Message};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    #[allow(dead_code)]
    ChannelsLoaded(Vec<Channel>),
    #[allow(dead_code)]
    MessagesLoaded(Vec<Message>),
    #[allow(dead_code)]
    Error(String),
}

pub fn spawn_event_loop(tx: mpsc::UnboundedSender<AppEvent>) {
    std::thread::spawn(move || {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(250)).unwrap_or(false)
                && let Ok(Event::Key(key)) = crossterm::event::read()
                && tx.send(AppEvent::Key(key)).is_err()
            {
                break;
            } else if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
}
