use std::io::{Read, Write};

fn handle_client(mut stream: std::net::TcpStream, address: std::net::SocketAddr) {
    // Spawn a thread to handle the new client
    std::thread::spawn(move || {
        println!("[Server] New client connection from {0}", address);

        let mut client_connected: bool = true;
        let mut data = [0 as u8; 512]; // using 512 byte buffer

        match stream.set_nonblocking(false) {
            Ok(_) => {
                println!(
                    "[Server] ({0}) Switched client to non-blocking mode.",
                    address
                );
            }
            Err(e) => {
                println!(
                    "[Server] ({0}) Failed to switch to blocking mode for client. Error: {1}",
                    address, e
                );
            }
        }

        while client_connected {
            // Try to read from stream
            match stream.read(&mut data) {
                Ok(0) => {
                    println!("[Server] ({0}) Disconnected.", address);
                    client_connected = false;
                }
                Ok(size) => {
                    let msg = std::str::from_utf8(&data[0..size]).unwrap();
                    println!("[Server] ({0}) Received {1} bytes: {2}", address, size, msg);

                    // Reply with quoted message
                    let reply = format!("Thank you for saying: {0}", msg);
                    stream.write(reply.as_bytes()).expect("Error sending reply.");
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

fn main() {
    if std::env::args().len() != 3 {
        println!("Usage: rusttcpclient ip port");
        return;
    }

    // Parse command-line arguments
    let ip = std::env::args()
        .nth(1)
        .expect("Expected argument 1 to be IP.");
    let port = std::env::args()
        .nth(2)
        .expect("Expected argument 2 to be port.");

    // Print out startup information
    println!("TCP Server");
    println!("~~~~~~~~~~");
    println!();
    println!("IP: {0}", ip);
    println!("Port: {0}", port);

    let addr = format!("{0}:{1}", ip, port);

    println!("Server address: {0}", addr);

    // Bind to the server address
    let listener = std::net::TcpListener::bind(addr).unwrap();

    // Set to non-blocking mode
    listener
        .set_nonblocking(true)
        .expect("[Server] Cannot set non-blocking mode.");

    println!("[Server] Listening for connections.");

    // Start server loop
    loop {
        // Check for an incoming connection
        match listener.accept() {
            Ok((stream, address)) => {
                handle_client(stream, address);
            }
            // Handle case where waiting for accept would become blocking
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                //println!("[Server] No incoming connections found.");
            }
            Err(e) => {
                println!("[Server] Error accepting client connection: {0}", e);
            }
        }
        // println!("[Server] Sleeping for 2 seconds.");
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }

    // println!("[Server] Quitting");
}
