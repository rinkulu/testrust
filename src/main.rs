use clap::Parser;
use ftail::Ftail;
use log::{LevelFilter, debug, error, info};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

mod commands;
mod handler;
mod types;

#[derive(Parser)]
#[command(version, about = None, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,

    /// Sets a custom log file
    #[arg(short, long, value_name = "FILE", default_value = "default.log")]
    log_file: PathBuf,
}

#[tokio::main]
async fn main() {
    // parsing arguments
    let cli = Cli::parse();
    let loglevel = match cli.debug {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };
    let logfile = cli.log_file.as_path();

    // setting up the logger
    if let Err(e) = Ftail::new()
        .single_file(logfile, true, loglevel)
        .timezone(ftail::Tz::UTC)
        .init()
    {
        eprintln!("Couldn't initialize the logger: {e}");
        return;
    }

    // setting up the listener
    let server_addr = "localhost:7878";
    let listener = match TcpListener::bind(server_addr).await {
        Ok(v) => v,
        Err(e) => {
            error!("Couldn't start the server: {e}");
            return;
        }
    };
    let mut tasks = JoinSet::new();

    info!("Server started on {server_addr}, ready to accept connections.");
    println!(
        "Server started on {server_addr}. Logs are available at {}",
        logfile.display()
    );

    // accepting connections
    loop {
        tokio::select! {
            conn = listener.accept() => {
                let (socket, addr) = match conn {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Couldn't accept an incoming connection: {e}");
                        continue;
                    }
                };
                debug!("Accepted incoming connection from {addr}.");
                tasks.spawn(async move {
                    handler::handle_connection(socket).await;
                });
            }
            sigint = tokio::signal::ctrl_c() => {
                if let Err(e) = sigint {
                    error!("Failed to set up Ctrl+C signal handler: {e}");
                    error!("Shutting down the server now since we are unable to receive stop signals properly.");
                    eprintln!("Unexpected error occurred: unable to listen for Ctrl+C signal. Stopping the server...");
                }
                else {
                    info!("Shutdown signal received, stopping acceptance of new connections.");
                    println!("Stopping the server...");
                }
                break;
            }
        }
    }

    info!("Waiting for existing connections to finish...");
    tasks.join_all().await;
    info!("Server shut down.");
    println!("Server stopped.");
}
