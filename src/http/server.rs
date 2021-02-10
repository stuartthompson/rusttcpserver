use std::sync::mpsc::TryRecvError;
use logger::{self, Log};
use super::client::{self, HttpClient};

// struct Client {
//     address: String,
//     client_to_server_rx: std::sync::mpsc::Receiver<String>,
//     server_to_client_tx: std::sync::mpsc::Sender<String>
// }

/**
 * Represents an HTTP server.
 */
struct HttpServer {
    address: String,
    name: String,
    is_admin_server: bool,
    main_to_server_rx: std::sync::mpsc::Receiver<String>,
    server_to_main_tx: std::sync::mpsc::Receiver<String>
}

/**
 * Starts an HTTP server.
 */
pub fn start(
    address: String,
    name: String,
    is_admin: bool,
    main_rx: std::sync::mpsc::Receiver<String>,
) -> std::sync::mpsc::Receiver<String> {
    // Logger
    let logger = logger::Logger { source: String::from(&name) };

    // Listener
    let listener =
        std::net::TcpListener::bind(&address).expect("[Server] Error binding TCP listener.");
    // Create a channel to communicate back to main thread
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // Set to non-blocking mode
    match listener.set_nonblocking(true) {
        Ok(_) => {
            logger.log_color("Set to non-blocking mode.", "red");
        }
        Err(err) => {
            println!(
                "[Server] ({0}) Could not set non-blocking mode. Error: {1}",
                name, err
            );
        }
    };

    // Start listener thread
    std::thread::spawn(move || {
        println!("[Server] ({0}) listening on {1}", name, &address);

        let mut server_running: bool = true;
        let mut clients: Vec<HttpClient> = Vec::new();

        while server_running {
            // Check for an incoming connection
            match listener.accept() {
                Ok((stream, address)) => {
                    let (client_to_server_tx, client_to_server_rx) = std::sync::mpsc::channel::<String>();
                    let (server_to_client_tx, server_to_client_rx) = std::sync::mpsc::channel::<String>();
                    
                    let client = HttpClient { 
                        address: address, 
                        client_to_server_rx: client_to_server_rx,
                        server_to_client_tx: server_to_client_tx,
                        is_connected: false
                    };
                    clients.push(client);

                    client::handle_client(stream, address, client_to_server_tx, server_to_client_rx);
                }
                // Handle case where waiting for accept would become blocking
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    println!(
                        "[Server] ({0}) Error accepting client connection. Error: {1}",
                        name, e
                    );
                }
            }

            // Check for notifications from existing clients
            for client in &clients {
                match client.client_to_server_rx.try_recv() {
                    Ok(message) => {
                        println!(
                            "[{0}] ({1}) Received message from client. Message: {2}",
                            name, client.address, message
                        );
                        if is_admin {
                            // Parse admin commands
                            parse_admin_command(message, &tx);
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!(
                        "[Server] Error reading from client channel receiver. Channel disconnected."
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
            println!("[Server] ({0}) Sending disconnect request to client at address {1}.", name, client.address);
            client.server_to_client_tx.send(String::from("Disconnect")).expect("[Server] ({0}) Error telling client to disconnect.");
        }

        while connected_clients > &disconnects {
            for client in &clients {
                match client.client_to_server_rx.try_recv() {
                    Ok(message) => {
                        if message == "Disconnected" {
                            println!("[{0}] Client @ {1} disconnected.", name, client.address);
                            disconnects = disconnects + 1;
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {},
                }
            }
            println!("Waiting for client disconnects.");
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        // Indicate to the main thread that this server has stopped
        tx.send(String::from("ServerStopped")).expect("[Server] Error sending ServerStopped message to main thread.");
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
