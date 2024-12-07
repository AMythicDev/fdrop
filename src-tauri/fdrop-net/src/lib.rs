mod definitions;
mod errors;

use definitions::{encode, LinkResponse, MessageType};
use errors::{CommunicationError, DiscoveryError, NetworkError};
use fdrop_config::UserConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use socket2::{Domain, Type};
use std::{
    collections::HashSet,
    hash::Hash,
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
    sync::Mutex,
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{info, warn};

const MDNS_SERVICE_TYPE: &str = "_fdrop._tcp.local.";
const FDROP_PORT: u16 = 10116;
const DEVICE_DISCOVERED: &str = "device-discovered";

#[derive(Debug)]
pub struct Connection {
    name: String,
    addresses: Vec<IpAddr>,
    stream: Option<TcpStream>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Connection {}

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
            stream: None,
        }
    }
}

impl Connection {
    async fn send_link_request(&mut self) -> Result<definitions::LinkResponse, CommunicationError> {
        if self.stream.is_some() {
            return Ok(LinkResponse::Accepted);
        }
        for addr in &self.addresses {
            let stream = TcpStream::connect(SocketAddr::new(addr.clone(), FDROP_PORT)).await;
            if let Ok(mut sock) = stream {
                let message = definitions::protobuf::Link {
                    request: Some(true),
                    response: None,
                };
                let auth_message = definitions::encode(MessageType::Link, message);
                sock.write_all(&*auth_message)
                    .await
                    .map_err(|e| CommunicationError::WriteError(e))?;
                let (_, _, _) = read_stream(&mut sock).await?;
                self.stream = Some(sock);
                return Ok(LinkResponse::Accepted);
            }
        }
        Err(CommunicationError::NoReachableAddress)
    }
}

pub struct ConnectionManager {
    mdns_daemon: ServiceDaemon,
    available_connections: HashSet<Connection>,
    instance_name: Option<String>,
}

impl ConnectionManager {
    pub fn new() -> Result<Mutex<Self>, NetworkError> {
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

async fn accept_connections() -> Result<(), CommunicationError> {
    let socket = socket2::Socket::new(Domain::IPV6, Type::STREAM, None)?;
    socket.set_only_v6(false)?;
    let address = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, FDROP_PORT, 0, 0);
    socket.bind(&address.into())?;
    socket.listen(128)?;
    let std_listener: std::net::TcpListener = socket.into();
    let listener: TcpListener = TcpListener::from_std(std_listener)?;
    info!("created the connection acceptor");

    tokio::spawn(async move {
        // TODO: look into error checking here
        // let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        // let mut connection_manager = cm_lock.lock().unwrap();
        loop {
            let conn = listener.accept().await;
            match conn {
                Ok((mut stream, _)) => {
                    handle_stream(&mut stream).await;
                }
                Err(e) => warn!("failed to connect to peer due to {e}"),
            }
        }
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

async fn read_stream(
    stream: &mut TcpStream,
) -> Result<(MessageType, u16, [u8; 2048]), CommunicationError> {
    const MAX_PAYLOAD_SIZE: usize = 2048;
    let mut payload = [0u8; MAX_PAYLOAD_SIZE];
    let mtype_u8 = stream
        .read_u8()
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    let mtype = MessageType::try_from(mtype_u8)?;
    let payload_size = stream
        .read_u16()
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    if payload_size > (MAX_PAYLOAD_SIZE as u16) {
        // Return a response with invalid payload error
        todo!();
    }
    stream
        .read_exact(&mut payload[0..(payload_size as usize)])
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    println!("{:?}", &payload[0..(payload_size as usize)]);
    Ok((mtype, payload_size, payload))
}

async fn handle_stream(stream: &mut TcpStream) -> Result<(), CommunicationError> {
    loop {
        let (mtype, _, _) = read_stream(stream).await?;
        match mtype {
            MessageType::Link => {
                let message = definitions::protobuf::Link {
                    request: None,
                    response: Some(LinkResponse::Accepted as i32),
                };
                stream
                    .write_all(&*encode(MessageType::Link, message))
                    .await?;
            }
        }
    }
}

pub mod commands {
    use super::*;
    #[tauri::command]
    pub async fn enable_networking(handle: AppHandle) -> Result<(), String> {
        launch_discovery_service(handle).map_err(|e| NetworkError::from(e))?;
        accept_connections()
            .await
            .map_err(|e| NetworkError::from(e))?;
        Ok(())
    }

    #[tauri::command]
    pub async fn link_device_by_name(handle: AppHandle, name: String) -> Result<(), String> {
        let mut actual_connection = {
            let cm_lock = handle.state::<Mutex<ConnectionManager>>();
            let mut connection_manager = cm_lock.lock().unwrap();
            let fake_connection = Connection {
                name,
                addresses: vec![],
                stream: None,
            };
            // TODO: Handle error
            connection_manager
                .available_connections
                .take(&fake_connection)
                .unwrap()
        };
        actual_connection
            .send_link_request()
            .await
            .map_err(|e| NetworkError::from(e))?;
        let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        let mut connection_manager = cm_lock.lock().unwrap();
        connection_manager
            .available_connections
            .insert(actual_connection);
        Ok(())
    }
}
