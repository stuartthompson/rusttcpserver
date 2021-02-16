use super::request;
use super::tcp_client_handler::{TcpClientAction, TcpClientRequestHandler};
use log::{debug, warn};
use std::io::Write;

pub struct HttpClientRequestHandler {
    //pub stream: &std::net::TcpStream,
    /**
     * IP address of connected client.
     */
    pub address: std::net::SocketAddr,
}

impl HttpClientRequestHandler {
    /**
     * Handles an HTTP client request.
     */
    fn handle_http_request(self: &HttpClientRequestHandler, mut stream: &std::net::TcpStream) {
        // Just respond with 200 OK
        let resp = b"HTTP/1.1 200 OK";
        match stream.write(resp) {
            Ok(_) => {
                debug!(
                    "[HTTP Client] ({0}) Sent response HTTP 200 OK",
                    self.address
                );
            }
            Err(error) => {
                debug!(
                    "[HTTP Client] ({0}) Error sending HTTP 200 OK response. {1}",
                    self.address, error
                );
            }
        }
    }
}

impl TcpClientRequestHandler for HttpClientRequestHandler {
    /**
     * Handles an HTTP client request.
     */
    fn handle_request(
        self: &HttpClientRequestHandler,
        stream: &std::net::TcpStream,
        data: &[u8],
        num_bytes: &usize,
    ) -> TcpClientAction {
        debug!(
            "[HTTP Client] ({0}) Received {1} bytes.",
            &self.address, num_bytes
        );

        match std::str::from_utf8(&data[0..*num_bytes]) {
            Ok(payload) => {
                // Parse the http request
                let request = request::parse_http_request(payload);

                // Is this a request to upgrade to a websocket?
                if request.connection == "Upgrade" && request.upgrade == "websocket" {
                    return TcpClientAction::UpgradeToWebSocket(request.sec_websocket_key);
                } else {
                    self.handle_http_request(stream);
                }
            }
            Err(error) => {
                warn!(
                    "[HTTP Client] ({0}) Error parsing client request to UTF-8: {1}",
                    self.address, error
                );
            }
        }

        return TcpClientAction::None;
    }
}
