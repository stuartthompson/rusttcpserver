use std::io::{Read, Write};
use crate::channel::Channel;
use super::response;
use std::sync::mpsc::TryRecvError;
use super::http_request_handler::HttpClientRequestHandler;
use super::websocket_request_handler::WebSocketClientRequestHandler;

pub struct TcpClientHandler {
    address: std::net::SocketAddr,
    is_connected: bool,
    client_type: TcpClientType,
    stream: std::net::TcpStream,
    server_channel: Channel<String>,
    request_handler: Box<dyn TcpClientRequestHandler + Send>
}

pub enum TcpClientType {
    Http,
    WebSocket
}

pub enum TcpClientAction {
    None,
    CloseConnection,
    UpgradeToWebSocket(String),
    RequestServerShutdown
}

pub trait TcpClientRequestHandler {
    fn handle_request(
        self: &Self, 
        stream: &std::net::TcpStream, 
        data: &[u8], 
        num_bytes: &usize) -> TcpClientAction;
}

impl TcpClientHandler {
    /**
     * Handles a new TCP client.
     */
    pub fn handle_new_client(
        stream: std::net::TcpStream,
        address: std::net::SocketAddr,
        client_type: TcpClientType,
        server_channel: Channel<String>
    ) {
        // Create the TCP client handler for this client
        let handler = TcpClientHandler {
            stream: stream,
            address: address,
            is_connected: false,
            client_type: client_type,
            server_channel: server_channel,
            request_handler: Box::new(HttpClientRequestHandler {
                address: address
            })
        };

        // Handle the client
        handler.handle_client();
    }

    /**
     * Begin handling communications with the TCP client.
     */
    fn handle_client(mut self: TcpClientHandler) {
        // Spawn a thread to handle the new client
        std::thread::spawn(move || {
            println!(
                "[TCP Client Handler] New client connection from {0}",
                &self.address
            );

            // Mark client as connected
            self.is_connected = true;
            self.server_channel
                .sender
                .send(String::from("Connected"))
                .expect("Error notifying server of client connection.");

            let mut buffer = [0 as u8; 4096];

            // Run while the client is connected
            while self.is_connected {
                match &self.stream.read(&mut buffer) {
                    Ok(0) => {
                        self.handle_disconnect();
                    }
                    Ok(size) => {
                        self.handle_request(&buffer, &size);
                    }
                    // Handle case where waiting for accept would become blocking
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    // Handle error case
                    Err(error) => {
                        self.handle_error(&error);
                        // Mark the client as disconnected
                        self.is_connected = false;
                    }
                }

                // Check for messages from server
                match self.server_channel.receiver.try_recv() {
                    Ok(message) => {
                        if message == "Disconnect" {
                            println!(
                                "[Client @ {0}] Received notification from server to disconnect.",
                                self.address
                            );
                            self.stream
                                .shutdown(std::net::Shutdown::Both)
                                .expect("Failed to shutdown client.");
                            // Mark the client as disconnected
                            self.is_connected = false;
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        println!("[Client @ {0}] Error receiving from server.", self.address);
                    }
                }

                // Sleep for a short time (let the client do something)
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Finalize disconnect
            self.server_channel.sender.send(String::from("Disconnected")).expect("Error notifying server that client disconnected.");
        });        
    }

    /**
     * Handles client disconnect.
     */
    fn handle_disconnect(&mut self) {
        println!("[TCP Client Handler] ({0}) Disconnected.", &self.address);
        self.is_connected = false;
        self.server_channel
            .sender
            .send(String::from("Client Disconnect"))
            .expect("Error notifying server of client disconnect.");
    }

    /**
     * Handlers client requests.
     */
    fn handle_request(&mut self, data: &[u8], size: &usize) {
        match self.request_handler.handle_request(&self.stream, data, size) {
            TcpClientAction::None => {},
            TcpClientAction::CloseConnection => {
                self.handle_disconnect();
            }
            TcpClientAction::UpgradeToWebSocket(sec_websocket_key) => {
                self.handle_websocket_upgrade_request(
                    sec_websocket_key);
            }
            TcpClientAction::RequestServerShutdown => {
                self.server_channel
                    .sender
                    .send(String::from("ShutdownServer"))
                    .expect("Error notifying server of shutdown request.");
            }
        }
    }

    /**
     * Handles errors reading from the client TCP stream.
     */
    fn handle_error(self: &TcpClientHandler, error: &std::io::Error) {
        println!("[TCP Client Handler] ({0}) Error: {1}", &self.address, error);
        // Inform the server of the error
        self.server_channel
            .sender
            .send(String::from("Client Communication Error"))
            .expect("Error notifying server of client communication error.");
    }

    /**
     * Handles a WebSocket upgrade request.
     */
    fn handle_websocket_upgrade_request(
        &mut self,
        sec_websocket_key: String
    ) {
        println!(
            "[TCP Client Handler] ({0}) Received request from client to upgrade to WebSocket connection.",
            self.address
        );
        // Build http response to upgrade to websocket
        let response = response::upgrade_to_websocket(&sec_websocket_key);
        // Send response to client accepting upgrade request
        println!("[TCP Client Handler] ({0}) Sending response accepting request to upgrade to WebSocket connection.", self.address);
        self.stream
            .write(response.as_bytes())
            .expect("Error sending response to upgrade to websocket.");

        // Communicate to server that connection has upgraded to WebSocket
        self.server_channel
            .sender
            .send(String::from("Upgrade to WebSocket"))
            .expect("Error notifying server of WebSocket upgrade.");

        // Replace the request handler with a websocket handler
        let websocket_handler = WebSocketClientRequestHandler {
            address: self.address
        };
        self.request_handler = Box::new(websocket_handler);
    }

    
}
