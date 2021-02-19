use std::io::{Read, Write};
use std::sync::mpsc::{TryRecvError, Sender, Receiver};
use log::{debug, warn};
use super::response;
use super::http_request_handler::HttpClientRequestHandler;
use super::websocket_request_handler::WebSocketClientRequestHandler;

pub struct TcpClientHandler {
    address: std::net::SocketAddr,
    is_connected: bool,
    client_type: TcpClientType,
    stream: std::net::TcpStream,
    to_server_tx: Sender<String>,
    from_server_rx: Receiver<String>,
    request_handler: Box<dyn TcpClientRequestHandler + Send>
}

pub enum TcpClientType {
    Http,
    WebSocket
}

pub enum TcpClientAction {
    None,
    HandleMessage(String),
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

    fn send_response(
        self: &Self, 
        stream: &mut std::net::TcpStream,
        message: String);
}

impl TcpClientHandler {
    /**
     * Handles a new TCP client.
     */
    pub fn handle_new_client(
        stream: std::net::TcpStream,
        address: std::net::SocketAddr,
        client_type: TcpClientType,
        to_server_tx: Sender<String>,
        from_server_rx: Receiver<String>,
    ) {
        // Create the TCP client handler for this client
        let handler = TcpClientHandler {
            stream: stream,
            address: address,
            is_connected: false,
            client_type: client_type,
            to_server_tx,
            from_server_rx,
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
            debug!(
                "[TCP Client Handler] New client connection from {0}",
                &self.address
            );

            debug!("[Client at {0}] Setting stream to non-blocking.", self.address);
            self.stream.set_nonblocking(true).expect("Error setting stream to non-blocking.");

            // Mark client as connected
            self.is_connected = true;
            self.to_server_tx
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
                match self.from_server_rx.try_recv() {
                    Ok(message) => {
                        debug!("[Client @ {0}] Received message from server: {1}.", self.address, message);

                        if message == "Disconnect" {
                            debug!(
                                "[Client @ {0}] Received notification from server to disconnect.",
                                self.address
                            );
                            self.stream
                                .shutdown(std::net::Shutdown::Both)
                                .expect("Failed to shutdown client.");
                            // Mark the client as disconnected
                            self.is_connected = false;
                        }

                        if message == "Send" {
                            debug!("[Client @ {0}] Received notification from server to send a message.", self.address);
                            (*self.request_handler).send_response(&mut self.stream, String::from("Bazinga!"));
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        warn!("[Client @ {0}] Error receiving from server.", self.address);
                    }
                }

                // Sleep for a short time (let the client do something)
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Finalize disconnect
            self.to_server_tx.send(String::from("Disconnected")).expect("Error notifying server that client disconnected.");
        });        
    }

    /**
     * Handles client disconnect.
     */
    fn handle_disconnect(&mut self) {
        debug!("[TCP Client Handler] ({0}) Disconnected.", &self.address);
        self.is_connected = false;
        self.to_server_tx
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
            TcpClientAction::HandleMessage(message) => {
                self.handle_message(message);
            }
            TcpClientAction::UpgradeToWebSocket(sec_websocket_key) => {
                self.handle_websocket_upgrade_request(
                    sec_websocket_key);
            }
            TcpClientAction::RequestServerShutdown => {
                debug!("[TCP Client Handler] ({0}): Received ShutdownServer request from handler.", self.address);
                self.to_server_tx
                    .send(String::from("ShutdownServer"))
                    .expect("Error notifying server of shutdown request.");
            }
        }
    }

    fn handle_message(&mut self, message: String) {
        self.to_server_tx.send(message).expect("Error notifying server of received message.");
    }

    /**
     * Handles errors reading from the client TCP stream.
     */
    fn handle_error(self: &TcpClientHandler, error: &std::io::Error) {
        warn!("[TCP Client Handler] ({0}) Error: {1}", &self.address, error);
        // Inform the server of the error
        self.to_server_tx
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
        debug!(
            "[TCP Client Handler] ({0}) Received request from client to upgrade to WebSocket connection.",
            self.address
        );
        // Build http response to upgrade to websocket
        let response = response::upgrade_to_websocket(&sec_websocket_key);
        // Send response to client accepting upgrade request
        debug!("[TCP Client Handler] ({0}) Sending response accepting request to upgrade to WebSocket connection.", self.address);
        self.stream
            .write(response.as_bytes())
            .expect("Error sending response to upgrade to websocket.");

        // Communicate to server that connection has upgraded to WebSocket
        self.to_server_tx
            .send(String::from("Upgrade to WebSocket"))
            .expect("Error notifying server of WebSocket upgrade.");

        // Replace the request handler with a websocket handler
        let websocket_handler = WebSocketClientRequestHandler {
            address: self.address
        };
        self.request_handler = Box::new(websocket_handler);
    }

    
}
