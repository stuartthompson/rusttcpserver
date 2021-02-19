use std::sync::mpsc::Sender;

pub trait Server {
    fn on_client_connected(self: &Self, sender: Sender<String>);
    fn on_message_received(self: &Self, client_id: &str, message: String);
}