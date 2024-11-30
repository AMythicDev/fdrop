use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tracing::info;

const MDNS_SERVICE_TYPE: &str = "_fdrop._udp.local.";
const FDROP_PORT: u16 = 10116;

#[derive(thiserror::Error, Debug)]
pub enum DiscoveryError {
    #[error("failed to create mDNS service daemon")]
    ServiceDomainError(#[from] mdns_sd::Error),
    #[error("cannot determine system hostname")]
    HostnameError(std::io::Error),
}

pub struct Connection {
    name: String,
    addresses: Vec<IpAddr>,
}

impl From<&ServiceInfo> for Connection {
    fn from(value: &ServiceInfo) -> Self {
        Connection {
            // TODO: Get proper name
            name: value.get_fullname().to_string(),
            addresses: value.get_addresses().iter().map(|i| *i).collect(),
        }
    }
}

pub struct ConnectionManager {
    mdns_daemon: ServiceDaemon,
    available_connections: Vec<Connection>,
}

impl ConnectionManager {
    pub fn new() -> Result<Mutex<Self>, DiscoveryError> {
        let mdns = ServiceDaemon::new()?;
        Ok(Mutex::new(Self {
            mdns_daemon: mdns,
            available_connections: Vec::new(),
        }))
    }

    pub fn launch(
        cm_lock: Arc<Mutex<ConnectionManager>>,
        instance_name: &str,
    ) -> Result<(), DiscoveryError> {
        let hs = whoami::fallible::hostname().map_err(|e| DiscoveryError::HostnameError(e))?;
        let local_hostname = format!("{}.local.", hs);

        let cm_lock2 = cm_lock.clone();

        // TODO: look into error checking here
        let connection_manager = cm_lock.lock().unwrap();

        let service = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            instance_name,
            &local_hostname,
            "",
            FDROP_PORT,
            None,
        )?
        .enable_addr_auto();
        connection_manager.mdns_daemon.register(service)?;
        let receiver = connection_manager.mdns_daemon.browse(MDNS_SERVICE_TYPE)?;
        info!("successfully created mdns service daemon");

        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if info.get_hostname() == local_hostname {
                            continue;
                        }
                        let mut connection_manager = cm_lock2.lock().unwrap();
                        connection_manager
                            .available_connections
                            .push(Connection::from(&info));
                        info!("found device with name: {}", info.get_fullname());
                    }
                    ServiceEvent::SearchStopped(ss) if ss == MDNS_SERVICE_TYPE => {
                        break;
                    }
                    other_event => {
                        println!("Received other event: {:?}", &other_event);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), DiscoveryError> {
        self.mdns_daemon.stop_browse(MDNS_SERVICE_TYPE)?;
        self.mdns_daemon.shutdown()?;
        info!("closed mdns service daemon");
        Ok(())
    }
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
