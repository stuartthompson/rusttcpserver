use super::web_request_handler;

/**
 * Represents an HTTP client connected via TCP.
 */
struct HTTPClient {
    /**
     * IP address of connected client.
     */
    address: std::net::SocketAddr,

    /**
     * The TCP stream for this client.
     */
    stream: std::net::TcpStream,

    /**
     * Communication channel from this client to the server.
     */
    to_server_tx: std::sync::mpsc::Sender<String>,

    /**
     * Communication channel from the server to this client.
     */
    from_server_rx: std::sync::mpsc::Receiver<String>,

    /**
     * A flag indicating if this client is connected.
     */
    is_connected: bool
}

pub fn handle_client(
    self: &HTTPClient) {
    // Spawn a thread to handle the new client
    std::thread::spawn(move || {
        println!("[HTTP Client] New client connection from {0}", self.address);
    
        let mut buffer = [0 as u8; 4096];

        // Run while the client is connected
        while(self.is_connected) {
            self.check_stream();
        }
    }
}

/**
 * Checks an HTTP client stream for new requests.
 */
fn check_stream(self: &HTTPClient) {
    match stream.read(&mut data) {
        // Zero bytes means client disconnected
        Ok(0) => {
            println!("[HTTP Client] ({0}) Disconnected.", address);
            client_connected = false;
        }
        // Parse received bytes
        Ok(size) => {
            println!("[HTTP Client] ({0}) Received {1} bytes.", address, size);
            self.handle_request(&data, size);
        }
        // Handle case where waiting for accept would become blocking
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        // Handle error case
        Err(error) => {
            println!("[HTTP Client] ({0}) Error: {1}", address, error);
            client_connected = false;
        }
    }    
}

/**
 * Handles an HTTP client request.
 */
fn handle_request(
    self: &HTTPClient, 
    data: &mut [u8], 
    num_bytes: usize
) {
    match std::str::from_utf8(&data[0..size]) {
        Ok(payload) => {
            println!(
                "[HTTP Client] ({0}) Received {1} bytes:",
                address, size
            );
            // Parse the http request
            let request = parse_http_request(payload);

            // Is this a request to upgrade to a websocket?
            if request.connection == "Upgrade" && request.upgrade == "websocket"
            {
                self.handle_websocket_upgrade_request(&request);
            } else {
                self.handle_http_request(&request);
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
    self: &HTTPClient, 
    request: &HTTPRequest
) {
    // Just respond with 200 OK
    let resp = b"HTTP/1.1 200 OK";
    match self.stream.write(resp) {
        Ok() => {
            println!("[HTTP Client] ({0}) Sent response HTTP 200 OK", self.address);
        },
        Err(error) => {
            println!("[HTTP Client] ({0}) Error sending HTTP 200 OK response. {1}", self.address, error);
        }
    }
}

/**
 * Handles a WebSocket upgrade request.
 */
fn handle_websocket_upgrade_request(
    self: &HTTPClient,
    request: &HTTPRequest
) {
    println!("[HTTP Client] ({0}) Received request from client to upgrade to WebSocket connection.", address);
    // Build http response to upgrade to websocket
    let response = http::response::upgrade_to_websocket(
        request.sec_websocket_key,
    );
    // Send response to client accepting upgrade request
    println!("[HTTP Client] ({0}) Sending response accepting request to upgrade to WebSocket connection.", address);
    stream
        .write(response.as_bytes())
        .expect("Error sending response to upgrade to websocket.");

    // Communicate to server that connection has upgraded to WebSocket
    self.to_server_tx.send(String::from("Upgrade to WebSocket")).expect("Error notifying server of WebSocket upgrade.");
    // Mark original HTTP connection as disconnected (TCPStream will remain open)
    self.is_connected = false;
}