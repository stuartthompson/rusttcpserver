use super::tcp_client_handler::{TcpClientHandler, TcpClientType};
use crate::client_handler::ClientHandler;
use log::{debug, info, warn};
use std::sync::mpsc::{channel, TryRecvError, Sender, Receiver};

struct TcpClient {
    pub address: std::net::SocketAddr,
    pub client_type: TcpClientType,
    pub is_connected: bool,
    pub to_client_tx: Sender<String>,
    pub from_client_rx: Receiver<String>
}

/**
 * Represents a TCP server.
 */
pub struct TcpServer {
    pub address: String,
    pub name: String,
    pub handler: Box<dyn ClientHandler + Send>,
    pub main_to_server_rx: Receiver<String>,
    pub server_to_main_tx: Sender<String>,
}

impl TcpServer {
    /**
     * Starts an HTTP server.
     */
    pub fn start(self: TcpServer) {
        // Start listener thread
        std::thread::spawn(move || {
            // Listener
            let listener = std::net::TcpListener::bind(&self.address)
                .expect("[Server] Error binding TCP listener.");

            // Set to non-blocking mode
            match listener.set_nonblocking(true) {
                Ok(_) => {
                    debug!("Set to non-blocking mode.");
                }
                Err(err) => {
                    warn!("[Server] Could not set non-blocking mode. Error: {0}", err);
                }
            };

            debug!("[Server] ({0}) listening on {1}", &self.name, &self.address);

            let mut server_running: bool = true;
            let mut clients: Vec<TcpClient> = Vec::new();

            while server_running {
                // Check for an incoming connection
                match listener.accept() {
                    Ok((stream, address)) => {
                        let (client_to_server_tx, client_to_server_rx) =
                            channel::<String>();
                        let (server_to_client_tx, server_to_client_rx) =
                            channel::<String>();
                        
                        // Hand off to a new TCP client handler
                        TcpClientHandler::handle_new_client(
                            stream,
                            address,
                            TcpClientType::Http,
                            client_to_server_tx,
                            server_to_client_rx
                        );

                        // Define a tracking client (used by the server to passively keep track of the client)
                        let client = TcpClient {
                            address: address,
                            client_type: TcpClientType::Http,
                            is_connected: false,
                            to_client_tx: server_to_client_tx,
                            from_client_rx: client_to_server_rx
                        };

                        &clients.push(client);
                    }
                    // Handle case where waiting for accept would become blocking
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        warn!(
                            "[Server] ({0}) Error accepting client connection. Error: {1}",
                            self.name, e
                        );
                    }
                }

                // Check for notifications from clients
                for client in clients.iter_mut() {
                    match client.from_client_rx.try_recv() {
                        Ok(message) => {
                            debug!(
                                "[{0}] ({1}) Received message from client. Message: {2}",
                                self.name, client.address, message
                            );
                            // TODO: This should be a command parser (vs. if blocks)
                            if message == "Connected" {
                                // TODO: Mark the client as connected
                                client.is_connected = true;

                                // Notify the handler (external implementation handler) of the new client
                                (*self.handler).on_client_connected(&client.address.to_string());
                            }
                            
                            else if message == "Upgrade to WebSocket" {
                                // ** UPGRADE THE HANDLER TO WEBSOCKET **
                                client.client_type = TcpClientType::WebSocket;
                            }

                            // Check for shutdown command
                            else if message == "ShutdownServer" {
                                info!("[Server] Admin command ShutdownServer received. Notifying Shutdown.");
                                self.server_to_main_tx
                                    .send(String::from("Shutdown"))
                                    .expect("Error sending shutdown notification to main thread.");
                            }

                            else {
                                // Notify external implementation handler of message
                                (*self.handler).on_message_received(&client.address.to_string(), &message);
                            }
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => panic!(
                            "[Server] Error reading from client channel receiver. Channel disconnected."
                        ),
                    }
                }

                // Check for messages from main thread
                match self.main_to_server_rx.try_recv() {
                    Ok(message) => {
                        debug!(
                            "[Server] ({0}) Received message from main thread. Message: {1}",
                            self.name, message
                        );
                        if message == "Send" {
                            // TODO: Look up correct client. Also, send actual message (need to upgrade mpsc channel type)
                            debug!("[Server] ({0}) Instructing client at {1} to send message.", self.name, clients[0].address);
                            clients[0].to_client_tx.send(String::from("Send")).expect("Error sending message to client.");
                        }
                        if message == "StopServer" {
                            // Kill this server
                            server_running = false;
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        panic!("[Server] Error reading from main thread receiver. Channel disconnected")
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Shutdown clients
            let connected_clients = &clients.len();
            let mut disconnects = 0;
            for client in &clients {
                debug!(
                    "[Server] ({0}) Sending disconnect request to client at address {1}.",
                    self.name, client.address
                );
                client
                    .to_client_tx
                    .send(String::from("Disconnect"))
                    .expect("[Server] ({0}) Error telling client to disconnect.");
            }

            while connected_clients > &disconnects {
                for client in &clients {
                    match client.from_client_rx.try_recv() {
                        Ok(message) => {
                            if message == "Disconnected" {
                                debug!(
                                    "[{0}] Client @ {1} disconnected.",
                                    self.name, client.address
                                );
                                disconnects = disconnects + 1;
                            }
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {}
                    }
                }
                debug!("Waiting for client disconnects.");
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }

            // Indicate to the main thread that this server has stopped
            self.server_to_main_tx
                .send(String::from("ServerStopped"))
                .expect("[Server] Error sending ServerStopped message to main thread.");
        });
    }
}
