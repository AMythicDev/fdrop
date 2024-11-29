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

pub struct ConnectionManager {
    mdns_daemon: ServiceDaemon,
}

impl ConnectionManager {
    pub fn new(instance_name: &str) -> Result<Self, DiscoveryError> {
        let mdns = ServiceDaemon::new()?;
        let hs = whoami::fallible::hostname().map_err(|e| DiscoveryError::HostnameError(e))?;
        let local_hostname = format!("{}.local.", hs);

        let service = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            instance_name,
            &local_hostname,
            "",
            FDROP_PORT,
            None,
        )?
        .enable_addr_auto();
        let monitor = mdns.monitor()?;
        mdns.register(service)?;
        let receiver = mdns.browse(MDNS_SERVICE_TYPE)?;
        info!("successfully created mdns service daemon");

        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if info.get_hostname() == local_hostname {
                            continue;
                        }
                        println!("Resolved a new service: {}", info.get_fullname());
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

        std::thread::spawn(move || {
            while let Ok(ev) = monitor.recv() {
                println!("Daemon event: {:?}", ev);
            }
        });

        Ok(Self { mdns_daemon: mdns })
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
