extern crate base64;
extern crate sha1;

mod http;
mod httpparser;
mod server;
mod websocket;

use std::sync::mpsc::TryRecvError;

fn main() {
    // Verify startup arguments
    if std::env::args().len() != 4 {
        println!("Usage: rusttcpclient ip port adminport");
        return;
    }

    // Parse command-line arguments
    let ip = std::env::args()
        .nth(1)
        .expect("Expected argument 1 to be IP.");
    let port = std::env::args()
        .nth(2)
        .expect("Expected argument 2 to be port.");
    let admin_port = std::env::args()
        .nth(3)
        .expect("Expected argument 3 to be admin port.");

    // Print out startup information
    println!("TCP Server");
    println!("~~~~~~~~~~");
    println!();
    println!("IP: {0}", ip);
    println!("Port: {0}", port);
    println!("Admin Port: {0}", admin_port);

    // Channel to communicate with the servers
    let (main_to_client_tx, main_to_client_rx) = std::sync::mpsc::channel::<String>();
    let (main_to_admin_tx, main_to_admin_rx) = std::sync::mpsc::channel::<String>();

    // Start client server
    let client_address = format!("{0}:{1}", ip, port);
    let client_rx = server::start(client_address, String::from("Client Server"), false, main_to_client_rx);

    // Start admin server
    let admin_address = format!("{0}:{1}", ip, admin_port);
    let admin_rx = server::start(admin_address, String::from("Admin Server"), true, main_to_admin_rx);

    // Server running flag
    let mut server_running = true;

    while server_running {
        // Check for messages from client server
        match client_rx.try_recv() {
            Ok(message) => {
                println!("[Main] Client server said: {0}", message);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("[Main] Error receiving on client server channel. Channel disconnected")
            }
        }

        // Check for messages from admin server
        match admin_rx.try_recv() {
            Ok(message) => {
                println!("[Main] Admin server said: {0}", message);

                // Check for shutdown
                if message == "Shutdown" {
                    server_running = false;
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("[Main] Error receiving on admin server channel. Channel disconnected")
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    
    // Tell servers to shut down
    println!("[Main] Sending shutdown message to client and admin servers."); 
    main_to_client_tx.send(String::from("Shutdown")).expect("[Main] Error communicating shutdown to client server.");
    main_to_admin_tx.send(String::from("Shutdown")).expect("[Main] Error communicating shutdown to admin server.");

    println!("[Main] Quitting");
}
