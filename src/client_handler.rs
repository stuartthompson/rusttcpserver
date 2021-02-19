use std::sync::mpsc::{Sender, Receiver};

pub trait ClientHandler {
    fn on_client_connected(self: &Self, sender: Sender<String>, receiver: Receiver<String>);
}