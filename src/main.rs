extern crate base64;
extern crate sha1;
extern crate colored;

mod http;
mod logger;
mod channel;

use std::sync::mpsc::TryRecvError;
use http::server::HttpServer;
use colored::*;

fn main() {
    // Print banner
    print_banner("WebRockets Server", "A simple HTTP/WebSockets server.");

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

    print_startup_info(&ip, &port, &admin_port);

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

fn print_banner(title: &str, description: &str) {
    // Calculate banner dimensions
    let banner_width = if title.len() > description.len() { title.len() } else { description.len() };
    let title_pad_width = (banner_width - title.len()) / 2;
    let desc_pad_width = banner_width - description.len();

    let banner_top = format!("┌{}┐", (0..banner_width + 3).map(|_| "─").collect::<String>()).magenta();
    let banner_bottom = format!("└{}┘", (0..banner_width + 3).map(|_| "─").collect::<String>()).magenta();
    let title_pad = (0..title_pad_width).map(|_| "~").collect::<String>().red();
    let empty_pad = (0..banner_width + 1).map(|_| " ").collect::<String>();
    let desc_pad = (0..desc_pad_width).map(|_| " ").collect::<String>();
    let lr_border = "│".magenta();

    println!();
    println!("{}", banner_top);
    println!("{} {} {} {} {}", lr_border, title_pad, title.yellow(), title_pad, lr_border);
    println!("{} {} {}", lr_border, empty_pad, lr_border);
    println!("{} {} {} {}", lr_border, description, desc_pad, lr_border);
    println!("{}", banner_bottom);
}

fn print_startup_info(ip: &str, port: &str, admin_port: &str) {
    let startup_panel_width = 35;

    let top = format!("┌{}┐", (0..startup_panel_width).map(|_| "─").collect::<String>()).green();
    let bottom = format!("└{}┘", (0..startup_panel_width).map(|_| "─").collect::<String>()).green();
    let edge = "│".green();

    println!("{}", top);
    println!("{} {} {} {}", edge, "Startup Parameters ".blue(), ("Startup Parameters ".len()+3..startup_panel_width).map(|_| " ").collect::<String>(), edge);
    println!("{}{}{}", edge, (0..startup_panel_width).map(|_| " ").collect::<String>(), edge);

    // IP address banner line
    let ipAddressBannerLine = BannerLine { 
        parts: vec![
            BannerPart { text: String::from("  IP Address: "), color: String::from("white")},
            BannerPart { text: String::from(ip), color: String::from("cyan") }
        ]
    };
    print_banner_line(ipAddressBannerLine, startup_panel_width);

    let adminPortBannerLine = BannerLine { 
        parts: vec![
            BannerPart { text: String::from("  Admin Port: "), color: String::from("white")},
            BannerPart { text: String::from(admin_port), color: String::from("cyan") }
        ]
    };
    print_banner_line(adminPortBannerLine, startup_panel_width);

    let publicPortBannerLine = BannerLine { 
        parts: vec![
            BannerPart { text: String::from(" Public Port: "), color: String::from("white")},
            BannerPart { text: String::from(port), color: String::from("cyan") }
        ]
    };
    print_banner_line(publicPortBannerLine, startup_panel_width);

    println!("{}", bottom);


    print!("{}", "test".color("blue"));
}

pub struct BannerLine {
    parts: Vec<BannerPart>
}

pub struct BannerPart {
    text: String,
    color: String
}

fn print_banner_line(line: BannerLine, panel_width: usize) {
    let edge = "│".green();
    let mut col: usize = 0;

    // Print left edge plus one space (col: 2)
    print!("{} ", edge);
    col = 1;

    // Print parts
    for part in line.parts.iter() {
        print!("{}", part.text.color(&part.color[..]));
        col = col + part.text.len();
    }

    // Print remaining space
    print!("{}", (col..panel_width).map(|_| " ").collect::<String>());
    println!("{}", edge);
}