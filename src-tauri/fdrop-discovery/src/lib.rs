use std::{
    collections::HashSet,
    hash::Hash,
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

#[derive(Debug, Eq)]
pub struct Connection {
    name: String,
    addresses: Vec<IpAddr>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // HACK: Hash strings until Rust feature `hasher_prefixfree_extras` isn't implemented
        state.write(self.name.as_bytes());
        state.write_u8(0xff);
    }
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
    available_connections: HashSet<Connection>,
}

impl ConnectionManager {
    pub fn new() -> Result<Mutex<Self>, DiscoveryError> {
        let mdns = ServiceDaemon::new()?;
        Ok(Mutex::new(Self {
            mdns_daemon: mdns,
            available_connections: HashSet::new(),
        }))
    }

    pub fn shutdown(&self) -> Result<(), DiscoveryError> {
        self.mdns_daemon.stop_browse(MDNS_SERVICE_TYPE)?;
        self.mdns_daemon.shutdown()?;
        info!("closed mdns service daemon");
        Ok(())
    }
}

pub mod commands {
    use tauri::{AppHandle, Manager};

    use super::*;
    #[tauri::command]
    pub fn launch_discovery_service(handle: AppHandle) -> Result<(), DiscoveryError> {
        let hs = whoami::fallible::hostname().map_err(|e| DiscoveryError::HostnameError(e))?;
        let local_hostname = format!("{}.local.", hs);

        let user_details = fdrop_config::commands::get_details_from_config(handle)?;

        // TODO: look into error checking here
        let connection_manager = handle.state::<Mutex<ConnectionManager>>().lock().unwrap();

        let service = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            &user_details.instance_name,
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
                        // TODO: look into error checking here
                        let mut connection_manager =
                            handle.state::<Mutex<ConnectionManager>>().lock().unwrap();
                        connection_manager
                            .available_connections
                            .replace(Connection::from(&info));
                        info!("found device with name: {}", info.get_fullname());
                        info!(
                            "Currnet devices: {:?}",
                            connection_manager.available_connections
                        );
                    }
                    ServiceEvent::SearchStopped(ss) if ss == MDNS_SERVICE_TYPE => {
                        break;
                    }
                    other_event => {
                        info!("Received other event: {:?}", &other_event);
                    }
                }
            }
        });
        Ok(())
    }
}
