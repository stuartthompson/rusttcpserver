use log::{debug};
use crate::server::Server;

struct MyServer {
    name: String   
}

impl Server for MyServer {
    fn on_client_connected(self: &ClientHandler, client_id: &str, sender: Sender<String>) {
        debug!("New client connected. Client id: {}", client_id);
    }

    fn on_message_received(self: &ClientHandler, client_id: &str, message: String) {
        debug!("Message received from client {}: {}", client_id, message)
    }
}