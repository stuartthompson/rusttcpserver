use std::sync::mpsc::{Sender, Receiver};
use log::{debug};
use crate::client_handler::ClientHandler;

pub struct MyServerImpl {
    name: String   
}

impl MyServerImpl {
    pub fn new(name: String) -> MyServerImpl {
        MyServerImpl { name }
    }
}

impl ClientHandler for MyServerImpl {
    /// Handles new client connections.
    /// 
    /// # Arguments
    /// 
    /// * `self` - The server handling the new client connection.
    /// * `client_id` - The unique id of the new client.
    /// * `sender` - A channel used to send messages to the newly connected client.
    fn on_client_connected(self: &Self, client_id: &str, sender: &Sender<String>) {
        debug!("(ExtImpl) New client connected. Client id: {}", client_id);
    }

    fn on_message_received(self: &Self, client_id: &str, message: &str) {
        debug!("(ExtImpl) Message received from client {}: {}", client_id, message);
    }
}