use std::io::Write;
use log::{debug};
use super::tcp_client_handler::{TcpClientAction, TcpClientRequestHandler};

pub struct WebSocketClientRequestHandler {
    /**
     * IP address of connected client.
     */
    pub address: std::net::SocketAddr,
}

impl TcpClientRequestHandler for WebSocketClientRequestHandler {
    /**
     * Handles an WebSocket client request.
     */
    fn handle_request(
        self: &WebSocketClientRequestHandler, 
        _stream: &std::net::TcpStream,
        data: &[u8], 
        num_bytes: &usize) -> TcpClientAction {
        debug!(
            "[WebSocket Client] ({0}) Received {1} bytes.",
            &self.address, num_bytes
        );

        // TODO: Move the following (commented) code to a trait for log display
        for i in 0..*num_bytes {
            println!("Byte {0: >2} is {1: >3}: {1:0>8b}", i, data[i]);
        }
        // Print the data as a base64 encoded string
        println!("Base64: {}", base64::encode(&data[0..*num_bytes]));
        
        let content = parse_websocket_frame(data, num_bytes);
        debug!("Received: {0}", content);
        
        // TODO: This should be a command-parser (vs. multiple if statement blocks)
        // Check for ShutdownServer command
        if content == "ShutdownServer" {
            return TcpClientAction::RequestServerShutdown;
        }

        return TcpClientAction::HandleMessage(content);
    }

    fn send_response(
        self: &WebSocketClientRequestHandler, 
        stream: &mut std::net::TcpStream,
        message: String) {
        // Build websocket frame
        let data: Vec<u8> = build_websocket_frame(&message.to_string());

        for i in 0..data.len() {
            println!("Byte {0: >2} is {1: >3}: {1:0>8b}", i, data[i]);
        }

        stream.write(&data).expect("Error writing message {0} to stream.");
    }
}

/// Returns the contents of a websocket frame.
/// 
/// WebSocket frame layout: https://tools.ietf.org/html/rfc6455#section-5.2
fn parse_websocket_frame(content: &[u8], size: &usize) -> String {
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

/// Returns a byte-array containing a websocket frame.
/// 
/// WebSocket frame layout: https://tools.ietf.org/html/rfc6455#section-5.2
/// 
/// # Arguments
/// 
/// * `content` - The content of the websocket frame.
fn build_websocket_frame(content: &str) -> Vec<u8> {
    let mut byte1: u8 = 0; // Flags and opcode
    let byte2: u8; // Mask and payload length

    // Set op code to 1 (text)
    byte1 |= 0b1000_0001;
    
    // TODO: Handle payloads longer than 126 bytes
    // Set payload length
    byte2 = content.len() as u8;

    let mut result: Vec<u8> = Vec::new();

    result.push(byte1);
    result.push(byte2);
    // result.push(masking_key[0]);
    // result.push(masking_key[1]);
    // result.push(masking_key[2]);
    // result.push(masking_key[3]);
    
    let c = content.as_bytes();
    for i in 0..content.len() {
        result.push(c[i]);
    }

    result
}