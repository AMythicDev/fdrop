use tracing::info;

#[tokio::main]
async fn main() {
    info!("logging enabled");
    fdrop_net::accept_connections().unwrap();
}
