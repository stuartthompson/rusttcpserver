use crate::client_handler::ClientHandler;
use crate::http::{Action, Request};
use log::debug;
use std::sync::mpsc::Sender;

pub struct MyServerImpl {
    name: String,
    to_server_tx: Sender<Request>,
}

impl MyServerImpl {
    pub fn new(name: String, to_server_tx: Sender<Request>) -> MyServerImpl {
        MyServerImpl { name, to_server_tx }
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
    fn on_client_connected(self: &Self, client_id: &str) {
        debug!("(ExtImpl) New client connected. Client id: {}", client_id);
    }

    fn on_message_received(self: &Self, client_id: &str, message: &str) {
        debug!(
            "(ExtImpl) Message received from client {}: {}",
            client_id, message
        );

        // Echo the message back
        self.to_server_tx.send(Request {
            client_id: String::from(client_id),
            action: Action::SendMessage(String::from(format!("Echo: {}", message))),
        }).expect("Error sending request to server.");
        //self.to_server_tx.send(String::from("Send"));
    }
}
