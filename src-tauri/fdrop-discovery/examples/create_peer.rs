#[tokio::main]
async fn main() {
    let key = libp2p::identity::ed25519::Keypair::generate();
    fdrop_discovery::peer_init_from_key(key).await.unwrap();
}
