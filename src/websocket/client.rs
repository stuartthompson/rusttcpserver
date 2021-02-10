use std::io::{Read, Write};

use std::sync::mpsc::TryRecvError;
use http;
use httpparser::parse_http_request;
use super::parser;

/**
 * Handles a websocket client.
 */
pub fn handle_client(
    mut stream: std::net::TcpStream, 
    address: std::net::SocketAddr, 
    to_server_tx: std::sync::mpsc::Sender<String>, 
    from_server_rx: std::sync::mpsc::Receiver<String>) {
    // Spawn a thread to handle the new client
    std::thread::spawn(move || {
        println!("[Server] New client connection from {0}", address);

        let mut client_connected: bool = true;
        let mut data = [0 as u8; 4096];

        let mut is_ws = false;
        while client_connected {
            // Try to read from stream
            match stream.read(&mut data) {
                Ok(0) => {
                    println!("[Websocket Client] ({0}) Disconnected.", address);
                    client_connected = false;
                }
                Ok(size) => {
                    println!("[Websocket Client] ({0}) Received {1} bytes.", address, size);
                    if is_ws {
                        // for i in 0..size {
                        //     println!("Byte {0: >2} is {1: >3}: {1:0>8b}", i, data[i]);
                        // }
                        let content = parser::parse_websocket_frame(data, size);
                        println!("Received: {0}", content);
                        // TODO: Parse messages from client (should be JSON)
                        // This should be command-parsing.
                        // Eliminate noise, only communication valid commands up the chain.
                        if content == "ShutdownServer" {
                            to_server_tx
                                .send(String::from("ShutdownServer"))
                                .expect("Error notifying server of shutdown request.");
                        }
                    } else {
                        match std::str::from_utf8(&data[0..size]) {
                            Ok(msg) => {
                                println!(
                                    "[Server] ({0}) Received {1} bytes:",
                                    address, size
                                );
                                // Reply with quoted message
                                let request = parse_http_request(msg);
                                if request.connection == "Upgrade" && request.upgrade == "websocket"
                                {
                                    // Build http response to upgrade to websocket
                                    let response = http::response::upgrade_to_websocket(
                                        request.sec_websocket_key,
                                    );
                                    println!("[Server] Sending response to upgrade connection.");
                                    stream
                                        .write(response.as_bytes())
                                        .expect("Error sending response to upgrade to websocket.");
                                    is_ws = true;
                                } else {
                                    let resp = b"HTTP/1.1 200 OK";
                                    stream
                                        .write(resp)
                                        .expect("Error responding with 200.");
                                }
                            }
                            Err(e) => {
                                println!("Error: {}", e);
                            }
                        }
                    }
                }
                // Handle case where waiting for accept would become blocking
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //println!("[Server] No incoming connections found.");
                }
                Err(error) => {
                    println!("[Server] ({0}) Error: {1}", address, error);
                    client_connected = false;
                }
            }

            // Check for messages from server
            match from_server_rx.try_recv() {
                Ok(message) => {
                    if message == "Disconnect" {
                        println!("[Client @ {0}] Received notification from server to disconnect.", address);
                        stream.shutdown(std::net::Shutdown::Both).expect("Failed to shutdown client.");
                        client_connected = false;
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("[Client @ {0}] Error receiving from server.", address);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        println!("[Client @ {0}] Notifying server of disconnect.", address);
        to_server_tx.send(String::from("Disconnected")).expect("Error notifying server that client disconnected.");

        println!("[Client @ {0}] Client disconnected.", address);
    });
}