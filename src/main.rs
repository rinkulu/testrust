use tokio::net::TcpListener;
mod handler;
mod types;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("localhost:7878").await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Couldn't start the server: {e}");
            return;
        }
    };
    loop {
        let (socket, _) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Couldn't accept an incoming connection: {e}");
                continue;
            }
        };
        tokio::spawn(async move {
            handler::handle_connection(socket).await;
        });
    }
}
