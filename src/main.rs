extern crate banner;
extern crate base64;
extern crate colored;
extern crate log4rs;
extern crate sha1;

mod client_handler;
mod http;
mod extimpl;

use banner::{Banner, Color, HeaderLevel, Style};
use extimpl::MyServerImpl;
use http::TcpServer;
use log::{debug, info, LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::sync::mpsc::TryRecvError;

fn main() {
    // Initialize logging
    init_logging().expect("Error initializing logger.");

    // Print banner
    print_title_banner();

    // Verify startup arguments
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
    
    print_startup_banner(&ip, &port);

    // Channel to communicate with the servers
    let (main_to_server_tx, main_to_server_rx) = std::sync::mpsc::channel::<String>();
    let (server_to_main_tx, server_to_main_rx) = std::sync::mpsc::channel::<String>();

    // Create client handler
    let my_server: MyServerImpl = MyServerImpl::new(String::from("MyServer"), main_to_server_tx);

    // Create server
    let server_address = format!("{0}:{1}", &ip, &port);
    let server: TcpServer = TcpServer {
        address: server_address,
        name: String::from("My Server"),
        handler: Box::new(my_server),
        main_to_server_rx: main_to_server_rx,
        server_to_main_tx: server_to_main_tx,
    };
    // Start server
    server.start();

    // Server running flag
    let mut server_running = true;

    while server_running {
        // Check for messages from server
        match server_to_main_rx.try_recv() {
            Ok(message) => {
                debug!("[Main] Server said: {0}", message);

                // Check for shutdown
                if message == "Shutdown" {
                    server_running = false;
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("[Main] Error receiving on server channel. Channel disconnected")
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    // Tell server to shut down
    info!("[Main] Sending shutdown message to server.");
    // main_to_server_tx
    //     .send(String::from("StopServer"))
    //     .expect("[Main] Error communicating shutdown to server.");

    let mut server_shutting_down = true;
    // Wait for server to shut down
    while server_shutting_down {
        match server_to_main_rx.try_recv() {
            Ok(message) => {
                if message == "ServerStopped" {
                    server_shutting_down = false;
                    debug!("[Main] Client server has stopped.");
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                debug!("[Main] Client server has disconnected.");
            }
        }
    }

    info!("[Main] Server stopped. Quitting.");
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
    info!("{}", banner.assemble());
}

fn print_startup_banner(ip: &str, port: &str) {
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
    banner.add_key_value("Port", port);

    info!("{}", banner.assemble());
}

fn init_logging() -> Result<(), SetLoggerError> {
    let file_path = "tmp/rusttcpserver.log";

    // Build a stderr logger
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Build a stdout logger
    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {M} {L}: {m} {f} Thread: {T}{I}\n",
        )))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Debug)))
                .build("stdout", Box::new(stdout)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Error)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config)?;

    Ok(())
}
