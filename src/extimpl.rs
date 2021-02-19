use std::sync::mpsc::{Sender, Receiver};
use log::{debug};
use crate::client_handler::ClientHandler;

struct MyServerImpl {
    name: String   
}

impl ClientHandler for MyServerImpl {
    /// Handles new client connections.
    /// 
    /// # Arguments
    /// 
    /// * `self` - The server handling the new client connection.
    /// * `client_id` - The unique id of the new client.
    /// * `sender` - A channel used to send messages to the newly connected client.
    /// * `receiver` - A channel used to receive messages from the newly connected client.
    fn on_client_connected(self: &Self, client_id: &str, sender: Sender<String>, receiver: Receiver<String>) {
        debug!("New client connected. Client id: {}", client_id);
    }
}