use std::sync::mpsc::Sender;

pub trait ClientHandler {
    fn on_client_connected(self: &Self, client_id: &str, sender: &Sender<String>);
    fn on_message_received(self: &Self, client_id: &str, message: &str);
}