use std::io::{Read, Write};

use http;
use httpparser::parse_http_request;

/**
 * Handles a websocket client.
 */
pub fn handle_client(mut stream: std::net::TcpStream, address: std::net::SocketAddr, tx: std::sync::mpsc::Sender<String>) {
    // Spawn a thread to handle the new client
    std::thread::spawn(move || {
        println!("[Server] New client connection from {0}", address);

        let mut client_connected: bool = true;
        let mut data = [0 as u8; 4096]; // using 512 byte buffer

        match stream.set_nonblocking(false) {
            Ok(_) => {
                println!("[Websocket Client] ({0}) Switched client to non-blocking mode.", address);
            }
            Err(e) => {
               println!(
                        "[Websocket Client] ({0}) Failed to switch to blocking mode for client. Error: {1}",
                        
                    address, e
                );
            }
        }

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
                        let content = parse_websocket_frame(data, size);
                        println!("Received: {0}", content);
                        // TODO: Parse messages from client (should be JSON)
                        // This should be command-parsing.
                        // Eliminate noise, only communication valid commands up the chain.
                        if content == "ShutdownServer" {
                            tx
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
                                println!("{0}", msg);
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
                Err(error) => {
                    println!("[Server] ({0}) Error: {1}", address, error);
                    client_connected = false;
                }
            }

            // Sleep for 100ms
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        println!("[Server] ({0}): Client disconnected.", address);
    });
}

/*
 * Parses a websocket frame.
 */
pub fn parse_websocket_frame(content: [u8; 4096], size: usize) -> String {
    let _fin_bit: bool = (content[0] & 0b10000000) != 0; // Bit 0 has fin bit
    let _rsv1: bool = (content[0] & 0b01000000) != 0; // Bit 1 contains reserved flag 1
    let _rsv2: bool = (content[0] & 0b00100000) != 0; // Bit 2 contains reserved flag 2
    let _rsv3: bool = (content[0] & 0b00010000) != 0; // Bit 3 contains reserved flag 3
    let _opcode = (content[0] & 0b00001111) as u8; // Bits 4 - 7 contain opcode (1-3 are reserved)
    let _mask_bit: bool = (content[1] & 0b10000000) != 0; // Bit 8 contains mask flag
    let _payload_len = (content[1] & 0b01111111) as u8; // Bits 9 - 15 contain payload length

    // TODO: Handle case where payload length > 126

    // Next 32-bits define the mask (in short payload case)
    let mask:[u8; 4] = [content[2], content[3], content[4], content[5]];

    // Decode payload content (XOR payload bits with mask bits)
    let mut decoded: Vec<u8> = Vec::new();
    for i in 0..size-6 {
        decoded.push(content[6+i] ^ mask[i % 4]); // 32 mask bits are used repeatedly
    }

    // Convert decoded payload into string
    let result = std::str::from_utf8(&decoded).expect("Error decoding websocket payload.");
    
    return String::from(result);
}