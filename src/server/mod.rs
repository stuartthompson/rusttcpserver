use websocket;

use std::sync::mpsc::TryRecvError;

/**
 * Starts a TCP server.
 */
pub fn start(
    address: String,
    name: String,
    is_admin: bool,
    main_rx: std::sync::mpsc::Receiver<String>,
) -> std::sync::mpsc::Receiver<String> {
    let listener =
        std::net::TcpListener::bind(address).expect("[Server] Error binding TCP listener.");
    // Create a channel to communicate back to main thread
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // Set to non-blocking mode
    match listener.set_nonblocking(true) {
        Ok(_) => {
            println!("[Server] ({0}) set to non-blocking mode.", name);
        }
        Err(err) => {
            println!(
                "[Server] ({0}) Could not set non-blocking mode. Error: {1}",
                name, err
            );
        }
    };

    // Vector to store clients
    // let mut clients: Vec<&Client> = Vec::new();

    // Start listener thread
    std::thread::spawn(move || {
        println!("[Server] ({0}) listening on .address.", name);

        let mut server_running: bool = true;
        let mut clients: Vec<std::sync::mpsc::Receiver<String>> = Vec::new();

        while server_running {
            // Check for an incoming connection
            match listener.accept() {
                Ok((stream, address)) => {
                    let (client_tx, client_rx) = std::sync::mpsc::channel::<String>();
                    websocket::client::handle_client(stream, address, client_tx);
                    clients.push(client_rx);
                }
                // Handle case where waiting for accept would become blocking
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //println!("[Server] No incoming connections found.");
                }
                Err(e) => {
                    println!(
                        "[Server] ({0}) Error accepting client connection. Error: {1}",
                        name, e
                    );
                }
            }

            // Check for notifications from existing clients
            for client_rx in &clients {
                match client_rx.try_recv() {
                    Ok(message) => {
                        println!(
                            "[Server] ({0}) Received message from client. Message: {1}",
                            name, message
                        );
                        if is_admin {
                            // Parse admin commands
                            parse_admin_command(message, &tx);
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!(
                        "[Server] Error reading from client channel receiver. Channel disconnected"
                    ),
                }
            }

            // Check for messages from main thread
            match main_rx.try_recv() {
                Ok(message) => {
                    println!(
                        "[Server] ({0}) Received message from main thread. Message: {1}",
                        name, message
                    );
                    if message == "Shutdown" {
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
    });

    return rx;
}

/**
 * Parses an admin command.
 */
fn parse_admin_command(command: String, main_tx: &std::sync::mpsc::Sender<String>) {
    // Check for shutdown command
    if command == "ShutdownServer" {
        println!("[Server] Admin command ShutdownServer received. Notifying Shutdown.");
        main_tx
            .send(String::from("Shutdown"))
            .expect("Error sending shutdown notification to main thread.");
    }
}
