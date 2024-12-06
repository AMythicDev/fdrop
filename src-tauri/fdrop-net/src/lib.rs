use fdrop_common::human_readable_error;
use fdrop_config::UserConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use socket2::{Domain, Type};
use std::{
    collections::HashSet,
    hash::Hash,
    io::Read,
    net::{IpAddr, Ipv6Addr, SocketAddrV6, TcpListener, TcpStream},
    sync::Mutex,
    thread,
};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

mod definitions;

const MDNS_SERVICE_TYPE: &str = "_fdrop._tcp.local.";
const FDROP_PORT: u16 = 10116;
const DEVICE_DISCOVERED: &str = "device-discovered";

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    #[error("discovery error")]
    DiscoveryError(#[from] DiscoveryError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DiscoveryError {
    #[error("service error")]
    ServiceError(mdns_sd::Error),
    #[error("failed to create mDNS service daemon")]
    ServiceDaemonError(mdns_sd::Error),
    #[error("failed to register service with mDNS service daemon")]
    ServiceRegisterError(mdns_sd::Error),
    #[error("failed to browse service with mDNS service daemon")]
    BrowseError(mdns_sd::Error),
    #[error("cannot determine system hostname")]
    HostnameError(std::io::Error),
    #[error("mDNS shutdown error")]
    ShutdownError(mdns_sd::Error),
    #[error(transparent)]
    TauriError(#[from] tauri::Error),
}

impl From<ConnectionError> for String {
    fn from(value: ConnectionError) -> Self {
        human_readable_error(&value)
    }
}

#[derive(Debug, Eq)]
pub struct Connection {
    name: String,
    addresses: Vec<IpAddr>,
    known: bool,
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
            known: false,
        }
    }
}

pub struct ConnectionManager {
    mdns_daemon: ServiceDaemon,
    available_connections: HashSet<Connection>,
    instance_name: Option<String>,
}

impl ConnectionManager {
    pub fn new() -> Result<Mutex<Self>, ConnectionError> {
        let mdns = ServiceDaemon::new().map_err(|e| DiscoveryError::ServiceDaemonError(e))?;
        Ok(Mutex::new(Self {
            mdns_daemon: mdns,
            available_connections: HashSet::new(),
            instance_name: None,
        }))
    }

    pub fn shutdown(&self) -> Result<(), DiscoveryError> {
        self.mdns_daemon
            .stop_browse(MDNS_SERVICE_TYPE)
            .map_err(|e| DiscoveryError::ShutdownError(e))?;
        if let Some(ref name) = self.instance_name {
            self.mdns_daemon
                .unregister(name)
                .map_err(|e| DiscoveryError::ShutdownError(e))?;
        }
        self.mdns_daemon
            .shutdown()
            .map_err(|e| DiscoveryError::ShutdownError(e))?;
        info!("closed mdns service daemon");
        Ok(())
    }

    pub fn get_availble_connections(&self) -> &HashSet<Connection> {
        &self.available_connections
    }
}

pub fn accept_connections() -> Result<(), ConnectionError> {
    let socket = socket2::Socket::new(Domain::IPV6, Type::STREAM, None)?;
    socket.set_only_v6(false)?;
    let address = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, FDROP_PORT, 0, 0);
    socket.bind(&address.into())?;
    socket.listen(128)?;
    let listener: TcpListener = socket.into();
    info!("created the connection acceptor");

    thread::spawn(move || -> Result<(), ConnectionError> {
        // TODO: look into error checking here
        // let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        // let mut connection_manager = cm_lock.lock().unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buff = [0; 1024];
                    let n = stream.read(&mut buff)?;
                    println!("{:?}", &buff[0..n]);
                }
                Err(e) => warn!("failed to connect to client with address due to {e}"),
            }
        }
        Ok(())
    });
    Ok(())
}

fn launch_discovery_service(handle: AppHandle) -> Result<(), DiscoveryError> {
    let hs = whoami::fallible::hostname().map_err(|e| DiscoveryError::HostnameError(e))?;
    let local_hostname = format!("{}.local.", hs);

    // TODO: look into error checking here
    let user_details_lock = handle.state::<Mutex<UserConfig>>();
    let user_details = user_details_lock.lock().unwrap();
    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
    let mut connection_manager = cm_lock.lock().unwrap();

    let service = ServiceInfo::new(
        MDNS_SERVICE_TYPE,
        &user_details.instance_name,
        &local_hostname,
        "",
        FDROP_PORT,
        None,
    )
    .map_err(|e| DiscoveryError::ServiceError(e))?
    .enable_addr_auto();
    connection_manager.instance_name = Some(service.get_fullname().to_string());
    connection_manager
        .mdns_daemon
        .register(service)
        .map_err(|e| DiscoveryError::ServiceDaemonError(e))?;
    let receiver = connection_manager
        .mdns_daemon
        .browse(MDNS_SERVICE_TYPE)
        .map_err(|e| DiscoveryError::BrowseError(e))?;
    info!("successfully created mdns service daemon");
    drop(connection_manager);
    drop(user_details);

    std::thread::spawn(move || -> Result<(), DiscoveryError> {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    if info.get_hostname() == local_hostname {
                        continue;
                    }
                    // TODO: look into error checking here
                    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
                    let mut connection_manager = cm_lock.lock().unwrap();
                    let con = Connection::from(&info);
                    handle.emit(DEVICE_DISCOVERED, &con.name)?;
                    connection_manager.available_connections.replace(con);
                    info!("found device with name: {}", info.get_fullname());
                }
                ServiceEvent::SearchStopped(ss) if ss == MDNS_SERVICE_TYPE => {
                    break;
                }
                ServiceEvent::SearchStarted(_) => {}
                other_event => {
                    info!("Received other event: {:?}", &other_event);
                }
            }
        }
        Ok(())
    });
    Ok(())
}

pub mod commands {
    use super::*;
    #[tauri::command]
    pub fn enable_networking(handle: AppHandle) -> Result<(), String> {
        launch_discovery_service(handle).map_err(|e| ConnectionError::from(e))?;
        accept_connections()?;
        Ok(())
    }
}
