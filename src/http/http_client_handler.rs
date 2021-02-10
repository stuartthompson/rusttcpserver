use super::{
    request::{self, HttpRequest},
    response,
};
use std::io::{Read, Write};
use std::sync::mpsc::TryRecvError;
use crate::channel::Channel;

/**
 * Represents an HTTP client handler.
 */
pub struct HttpClientHandler {
    /**
     * The TCP stream used to communicate with this client.
     */
    pub stream: std::net::TcpStream,

    /**
     * IP address of connected client.
     */
    pub address: std::net::SocketAddr,

    /**
     * A flag indicating if this client is connected.
     */
    pub is_connected: bool,

    /**
     * A channel used to communicate with the server.
     */
    pub server_channel: Channel,
}

impl HttpClientHandler {
    /**
     * Instantiates a new HttpClientHandler.
     */
    pub fn new(
        stream: std::net::TcpStream,
        address: std::net::SocketAddr,
        to_server_tx: std::sync::mpsc::Sender<String>,
        from_server_rx: std::sync::mpsc::Receiver<String>,
    ) -> HttpClientHandler {
        return HttpClientHandler {
            stream: stream,
            address: address,
            server_channel: Channel {
                sender: to_server_tx,
                receiver: from_server_rx,
            },
            is_connected: false,
        };
    }

    /**
     * Begin handling communications with the HTTP client.
     */
    pub fn handle_client(mut self: HttpClientHandler) {
        // Spawn a thread to handle the new client
        std::thread::spawn(move || {
            println!(
                "[HTTP Client] New client connection from {0}",
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
                        println!("[HTTP Client] ({0}) Disconnected.", &self.address);
                        self.is_connected = false;
                        self.server_channel
                            .sender
                            .send(String::from("Client Disconnect"))
                            .expect("Error notifying server of client disconnect.");
                    }
                    Ok(size) => {
                        self.handle_request(&mut buffer, &size);
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
     * Handles errors reading from the client TCP stream.
     */
    fn handle_error(self: &HttpClientHandler, error: &std::io::Error) {
        println!("[HTTP Client] ({0}) Error: {1}", &self.address, error);
        // Inform the server of the error
        self.server_channel
            .sender
            .send(String::from("Client Communication Error"))
            .expect("Error notifying server of client communication error.");
    }

    /**
     * Handles an HTTP client request.
     */
    fn handle_request(self: &HttpClientHandler, data: &mut [u8], num_bytes: &usize) {
        println!(
            "[HTTP Client] ({0}) Received {1} bytes.",
            &self.address, num_bytes
        );

        match std::str::from_utf8(&data[0..*num_bytes]) {
            Ok(payload) => {
                // Parse the http request
                let request = request::parse_http_request(payload);

                // Is this a request to upgrade to a websocket?
                if request.connection == "Upgrade" && request.upgrade == "websocket" {
                    self.handle_websocket_upgrade_request(&self.stream, &request);
                } else {
                    self.handle_http_request(&self.stream);
                }
            }
            Err(error) => {
                println!(
                    "[HTTP Client] ({0}) Error parsing client request to UTF-8: {1}",
                    self.address, error
                );
            }
        }
    }

    /**
     * Handles an HTTP request.
     */
    fn handle_http_request(self: &HttpClientHandler, mut stream: &std::net::TcpStream) {
        // Just respond with 200 OK
        let resp = b"HTTP/1.1 200 OK";
        match stream.write(resp) {
            Ok(_) => {
                println!(
                    "[HTTP Client] ({0}) Sent response HTTP 200 OK",
                    self.address
                );
            }
            Err(error) => {
                println!(
                    "[HTTP Client] ({0}) Error sending HTTP 200 OK response. {1}",
                    self.address, error
                );
            }
        }
    }

    /**
     * Handles a WebSocket upgrade request.
     */
    fn handle_websocket_upgrade_request(self: &HttpClientHandler, mut stream: &std::net::TcpStream, request: &HttpRequest) {
        println!(
            "[HTTP Client] ({0}) Received request from client to upgrade to WebSocket connection.",
            self.address
        );
        // Build http response to upgrade to websocket
        let response = response::upgrade_to_websocket(&request.sec_websocket_key);
        // Send response to client accepting upgrade request
        println!("[HTTP Client] ({0}) Sending response accepting request to upgrade to WebSocket connection.", self.address);
        stream
            .write(response.as_bytes())
            .expect("Error sending response to upgrade to websocket.");

        // Communicate to server that connection has upgraded to WebSocket
        self.server_channel
            .sender
            .send(String::from("Upgrade to WebSocket"))
            .expect("Error notifying server of WebSocket upgrade.");
    }
}
