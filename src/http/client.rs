use std::io::{Read, Write};
use std::sync::mpsc::TryRecvError;
use super::{request::{self, HttpRequest}, response};

/**
 * Represents an HTTP client handler thread.
 */
pub struct HttpClient {
    /**
     * IP address of connected client.
     */
    pub address: std::net::SocketAddr,

    /**
     * A flag indicating if this client is connected.
     */
    pub is_connected: bool,

    /**
     * Channel used by server to receive messages from client.
     */
    pub client_to_server_rx: std::sync::mpsc::Receiver<String>,

    /**
     * Channel used by server to transmit messages to client.
     */
    pub server_to_client_tx: std::sync::mpsc::Sender<String>
}

/**
 * Handles a new HTTP client. 
 */
pub fn handle_client(
    mut stream: std::net::TcpStream, 
    address: std::net::SocketAddr,
    to_server_tx: std::sync::mpsc::Sender<String>,
    from_server_rx: std::sync::mpsc::Receiver<String>) {
    // Spawn a thread to handle the new client
    std::thread::spawn(move || {
        println!("[HTTP Client] New client connection from {0}", &address);
    
        let mut buffer = [0 as u8; 4096];
        let mut is_connected: bool = true;
        
        // Run while the client is connected
        while is_connected {
            match &stream.read(&mut buffer) {
                Ok(0) => {
                    println!("[HTTP Client] ({0}) Disconnected.", &address);
                    is_connected = false;
                    to_server_tx
                        .send(String::from("Client Disconnect"))
                        .expect("Error notifying server of client disconnect.");
                }
                Ok(size) => {
                    handle_request(&stream, &address, &mut buffer, &size, &to_server_tx);
                }
                // Handle case where waiting for accept would become blocking
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                // Handle error case
                Err(error) => {
                    handle_error(&address, &error, &to_server_tx);
                    is_connected = false;
                }
            }
        }

        // Check for messages from server
        match from_server_rx.try_recv() {
            Ok(message) => {
                if message == "Disconnect" {
                    println!("[Client @ {0}] Received notification from server to disconnect.", &address);
                    stream.shutdown(std::net::Shutdown::Both).expect("Failed to shutdown client.");
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                println!("[Client @ {0}] Error receiving from server.", &address);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    // Finalize disconnect
    println!("[Client @ {0}] Notifying server of disconnect.", address);
    // to_server_tx.send(String::from("Disconnected")).expect("Error notifying server that client disconnected.");

    println!("[Client @ {0}] Client disconnected.", address);
}

fn handle_error(
    address: &std::net::SocketAddr,
    error: &std::io::Error,
    to_server_tx: &std::sync::mpsc::Sender<String>
) {
    println!("[HTTP Client] ({0}) Error: {1}", address, error);
    to_server_tx
        .send(String::from("Client Communication Error"))
        .expect("Error notifying server of client communication error.");
}

/**
 * Handles an HTTP client request.
 */
fn handle_request(
    stream: &std::net::TcpStream,
    address: &std::net::SocketAddr, 
    data: &mut [u8], 
    num_bytes: &usize,
    to_server_tx: &std::sync::mpsc::Sender<String>
) {
    println!("[HTTP Client] ({0}) Received {1} bytes.", &address, num_bytes);

    match std::str::from_utf8(&data[0..*num_bytes]) {
        Ok(payload) => {
            // Parse the http request
            let request = request::parse_http_request(payload);

            // Is this a request to upgrade to a websocket?
            if request.connection == "Upgrade" && request.upgrade == "websocket"
            {
                handle_websocket_upgrade_request(stream, address, &request, to_server_tx);
            } else {
                handle_http_request(stream, address);
            }
        }
        Err(error) => {
            println!("[HTTP Client] ({0}) Error parsing client request to UTF-8: {1}", address, error);
        }
    }
}

/**
 * Handles an HTTP request.
 */
fn handle_http_request(
    mut stream: &std::net::TcpStream, 
    address: &std::net::SocketAddr
) {
    // Just respond with 200 OK
    let resp = b"HTTP/1.1 200 OK";
    match stream.write(resp) {
        Ok(_) => {
            println!("[HTTP Client] ({0}) Sent response HTTP 200 OK", address);
        },
        Err(error) => {
            println!("[HTTP Client] ({0}) Error sending HTTP 200 OK response. {1}", address, error);
        }
    }
}

/**
 * Handles a WebSocket upgrade request.
 */
fn handle_websocket_upgrade_request(
    mut stream: &std::net::TcpStream,
    address: &std::net::SocketAddr,
    request: &HttpRequest,
    to_server_tx: &std::sync::mpsc::Sender<String>
) {
    println!("[HTTP Client] ({0}) Received request from client to upgrade to WebSocket connection.", address);
    // Build http response to upgrade to websocket
    let response = response::upgrade_to_websocket(
        &request.sec_websocket_key,
    );
    // Send response to client accepting upgrade request
    println!("[HTTP Client] ({0}) Sending response accepting request to upgrade to WebSocket connection.", address);
    stream
        .write(response.as_bytes())
        .expect("Error sending response to upgrade to websocket.");

    // Communicate to server that connection has upgraded to WebSocket
    to_server_tx.send(String::from("Upgrade to WebSocket")).expect("Error notifying server of WebSocket upgrade.");
}