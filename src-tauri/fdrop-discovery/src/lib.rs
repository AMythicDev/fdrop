// use libp2p::identity::ed25519::Keypair;
// use libp2p::{futures::StreamExt, mdns, swarm::SwarmEvent};
// use std::time::Duration;
// use tauri::AppHandle;
// use tokio::select;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

const MDNS_SERVICE_TYPE: &str = "_fdrop._udp.local.";
const FDROP_PORT: u16 = 10116;

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

pub fn create_publisher(instance_name: &str) {
    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let hs = hostname::get().unwrap();
    let local_hostname = format!("{}.local.", hs.to_str().unwrap());

    let service = ServiceInfo::new(
        MDNS_SERVICE_TYPE,
        instance_name,
        &local_hostname,
        "",
        FDROP_PORT,
        None,
    )
    .unwrap()
    .enable_addr_auto();
    let monitor = mdns.monitor().expect("Failed to monitor the daemon");
    mdns.register(service).unwrap();
    let receiver = mdns.browse(MDNS_SERVICE_TYPE).expect("Failed to browse");

    let t1 = std::thread::spawn(move || loop {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    if info.get_hostname() == local_hostname {
                        continue;
                    }
                    println!("Resolved a new service: {}", info.get_fullname());
                }
                other_event => {
                    println!("Received other event: {:?}", &other_event);
                }
            }
        }
    });

    let t2 = std::thread::spawn(move || {
        while let Ok(ev) = monitor.recv() {
            println!("Daemon event: {:?}", ev);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
}

pub mod commands {
    use fdrop_common::human_readable_error;

    use super::*;

    // #[tauri::command]
    // pub async fn create_peer(handle: AppHandle) -> Result<(), String> {
    //     let key = fdrop_config::read_keys(&handle)
    //         .map_err(|err| fdrop_common::human_readable_error(&err))?;
    //     peer_init_from_key(key)
    //         .await
    //         .map_err(|e| human_readable_error(&e))
    // }
}
