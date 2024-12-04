use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();
    info!("logging enabled");
    let cm = Arc::new(fdrop_discovery::ConnectionManager::new().unwrap());
    fdrop_discovery::ConnectionManager::launch(cm, "arijit-laptop").unwrap();
    loop {}
}
