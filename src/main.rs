use clap::Parser;
use ftail::Ftail;
use log::{LevelFilter, debug, error, info};
use std::path::PathBuf;
use tokio::net::TcpListener;

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

    // setting up the logger
    if let Err(e) = Ftail::new()
        .single_file(&cli.log_file, true, loglevel)
        .timezone(ftail::Tz::UTC)
        .init()
    {
        eprintln!("Couldn't initialize the logger: {e}");
        return;
    }

    // setting up the listener
    let listener = match TcpListener::bind("localhost:7878").await {
        Ok(v) => v,
        Err(e) => {
            error!("Couldn't start the server: {e}");
            return;
        }
    };
    info!("Server started.");

    // accepting connections
    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Couldn't accept an incoming connection: {e}");
                continue;
            }
        };
        debug!("Accepted incoming connection from {addr}.");
        tokio::spawn(async move {
            handler::handle_connection(socket).await;
        });
    }

    // TODO: graceful shutdown
    // info!("Server shut down.");
}
