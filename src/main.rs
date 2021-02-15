extern crate base64;
extern crate sha1;
extern crate colored;
extern crate banner;

mod http;
mod logger;
mod channel;

use std::sync::mpsc::TryRecvError;
use http::server::HttpServer;
use banner::{Banner, Style, HeaderLevel, Color};

fn main() {
    // Print banner
    print_title_banner();

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

    print_startup_banner(&ip, &port, &admin_port);

    // Logger
    let logger = logger::Logger { source: String::from("Main") };

    // Channel to communicate with the servers
    let (main_to_client_tx, main_to_client_rx) = std::sync::mpsc::channel::<String>();
    let (client_to_main_tx, client_to_main_rx) = std::sync::mpsc::channel::<String>();
    let (main_to_admin_tx, main_to_admin_rx) = std::sync::mpsc::channel::<String>();
    let (admin_to_main_tx, admin_to_main_rx) = std::sync::mpsc::channel::<String>();

    // Start client server
    let client_address = format!("{0}:{1}", &ip, &port);
    let client_server: HttpServer = HttpServer {
        address: client_address,
        name: String::from("Client Server"),
        is_admin_server: false,
        main_to_server_rx: main_to_client_rx,
        server_to_main_tx: client_to_main_tx
    };
    client_server.start();

    // Start admin server
    let admin_address = format!("{0}:{1}", &ip, &admin_port);
    let admin_server: HttpServer = HttpServer {
        address: admin_address,
        name: String::from("Admin Server"),
        is_admin_server: true,
        main_to_server_rx: main_to_admin_rx,
        server_to_main_tx: admin_to_main_tx
    };
    admin_server.start();

    // Server running flag
    let mut server_running = true;

    while server_running {
        // Check for messages from client server
        match client_to_main_rx.try_recv() {
            Ok(message) => {
                println!("[Main] Client server said: {0}", message);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("[Main] Error receiving on client server channel. Channel disconnected")
            }
        }

        // Check for messages from admin server
        match admin_to_main_rx.try_recv() {
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
    main_to_client_tx.send(String::from("StopServer")).expect("[Main] Error communicating shutdown to client server.");
    main_to_admin_tx.send(String::from("StopServer")).expect("[Main] Error communicating shutdown to admin server.");

    let mut client_server_running = true;
    let mut admin_server_running = true;

    // Wait for client and admin servers to stop
    while client_server_running || admin_server_running {
        if client_server_running {
            match client_to_main_rx.try_recv() {
                Ok(message) => {
                    if message == "ServerStopped" {
                        client_server_running = false;
                        println!("[Main] Client server has stopped.");
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("[Main] Client server has disconnected.");
                }
            }
        }

        if admin_server_running {
            match admin_to_main_rx.try_recv() {
                Ok(message) => {
                    if message == "ServerStopped" {
                        admin_server_running = false;
                        println!("[Main] Admin server has stopped.");
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("[Main] Admin server has disconnected.");
                }
            }
        }
    }

    println!("[Main] All servers stopped. Quitting.");
    std::thread::sleep(std::time::Duration::from_millis(3000));
}

fn print_title_banner() {
    // Create a style
    let mut style: Style = Style::new();
    style.border.color = Color::Red;
    style.h1.content_color = Color::Yellow;
    style.text.content_color = Color::White;
    
    // Create header banner
    let mut banner = Banner::new(&style);
    // TODO: Remove when default width is fixed
    banner.width = 0;

    // Add headers
    banner.add_header("WebRockets", HeaderLevel::H1);
    banner.add_text("A simple HTTP/WebSockets server.");

    // Print banner
    println!("{}", banner.assemble());
}

fn print_startup_banner(ip: &str, port: &str, admin_port: &str) {
    // Create a style
    let mut style: Style = Style::new();
    style.border.color = Color::Green;
    style.text.content_color = Color::Cyan;

    // Create startup params banner
    let mut banner: Banner = Banner::new(&style);
    // TODO: Remove when default width is fixed
    banner.width = 0;

    // Add params
    banner.add_header("Startup Parameters", HeaderLevel::H1);
    banner.add_key_value("IP Address", ip);
    banner.add_key_value("Admin Port", admin_port);
    banner.add_key_value("Public Port", port);

    println!("{}", banner.assemble());
}