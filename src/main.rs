use ftail::Ftail;
use log::{debug, error, info, LevelFilter};
use std::path::Path;
use tokio::net::TcpListener;

mod handler;
mod types;

#[tokio::main]
async fn main() {
    if let Err(e) = Ftail::new()
        .single_file(Path::new("default.log"), true, LevelFilter::Debug)
        .timezone(ftail::Tz::UTC)
        .init()
    {
        eprintln!("Couldn't initialize the logger: {e}");
        return;
    }

    let listener = match TcpListener::bind("localhost:7878").await {
        Ok(v) => v,
        Err(e) => {
            error!("Couldn't start the server: {e}");
            return;
        }
    };
    info!("Server started.");

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
}
