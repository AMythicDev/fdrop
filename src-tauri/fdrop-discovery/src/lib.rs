// use libp2p::identity::ed25519::Keypair;
// use libp2p::{futures::StreamExt, mdns, swarm::SwarmEvent};
// use std::time::Duration;
// use tauri::AppHandle;
// use tokio::select;

// #[derive(thiserror::Error, Debug)]
// pub enum DiscoveryError {
//     #[error(transparent)]
//     KeyReadError(fdrop_config::ConfigError),
//     #[error("Failed to listen on network")]
//     NetworkError(libp2p::TransportError<std::io::Error>),
// }

// pub async fn peer_init_from_key(key: Keypair) -> Result<(), DiscoveryError> {
//     tracing::info!("building the swarn instance");
//     let mut swarn = libp2p::SwarmBuilder::with_existing_identity(key.into())
//         .with_tokio()
//         .with_quic()
//         .with_behaviour(|key| {
//             let mdns =
//                 mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
//             Ok(mdns)
//         })
//         .unwrap()
//         .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
//         .build();

//     tracing::info!("trying to listen on address 0.0.0.0");
//     swarn
//         .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
//         .map_err(|e| DiscoveryError::NetworkError(e))?;
//     tracing::info!("successfully connected to the network");

//     loop {
//         select!(
//             event = swarn.select_next_some() => match event {
//                 SwarmEvent::Behaviour(mdns::Event::Discovered(list)) => {
//                     for (peer_id, _multiaddr) in list {
//                         println!("mDNS discovered a new peer: {peer_id}");
//                     }
//                 },
//                 SwarmEvent::Behaviour(mdns::Event::Expired(list)) => {
//                     for (peer_id, _multiaddr) in list {
//                         println!("mDNS discover peer has expired: {peer_id}");
//                     }
//                 },
//                 _ => {},
//             }
//         )
//     }
// }

// pub mod commands {
//     use fdrop_common::human_readable_error;

//     use super::*;

//     #[tauri::command]
//     pub async fn create_peer(handle: AppHandle) -> Result<(), String> {
//         let key = fdrop_config::read_keys(&handle)
//             .map_err(|err| fdrop_common::human_readable_error(&err))?;
//         peer_init_from_key(key)
//             .await
//             .map_err(|e| human_readable_error(&e))
//     }
// }
