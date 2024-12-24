mod definitions;
mod errors;

use bytes::{Bytes, BytesMut};
use definitions::{LinkResponse, TransferType};
use errors::{CommunicationError, DiscoveryError, NetworkError};
use fdrop_config::UserConfig;
use flume::{bounded, Receiver, Sender};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use prost::Message;
use socket2::{Domain, Type};
use std::{
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tracing::{error, info, warn};

const MDNS_SERVICE_TYPE: &str = "_fdrop._tcp.local.";
const FDROP_PORT: u16 = 10116;
const DEVICE_DISCOVERED: &str = "device-discovered";
const DEVICE_REMOVED: &str = "device-removed";
const LINK_RESPONSE: &str = "link-response";
const DEVICE_LINKED: &str = "device-linked";
const MAX_PAYLOAD_SIZE: usize = 2048;

#[derive(Debug)]
pub struct Connection {
    pub info: ConnectionInfo,
    addresses: Vec<IpAddr>,
    tx: Option<Sender<Bytes>>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ConnectionInfo {
    pub name: String,
    pub linked: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct Transfer {
    ttype: TransferType,
    display_content: String,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.info.name == other.info.name
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // HACK: Hash strings until Rust feature `hasher_prefixfree_extras` isn't implemented
        state.write(self.info.name.as_bytes());
        state.write_u8(0xff);
    }
}

impl From<&ServiceInfo> for Connection {
    fn from(value: &ServiceInfo) -> Self {
        let info = ConnectionInfo {
            // TODO: Get proper name
            name: value.get_fullname().to_string(),
            linked: false,
        };
        Connection {
            info,
            addresses: value.get_addresses().iter().map(|i| *i).collect(),
            tx: None,
        }
    }
}

impl Connection {
    #[tracing::instrument(skip(handle))]
    async fn send_link_request(
        &mut self,
        handle: AppHandle,
        our_name: &str,
    ) -> Result<LinkResponse, CommunicationError> {
        if self.tx.is_some() {
            return Ok(LinkResponse::Accepted);
        }
        for addr in &self.addresses {
            let stream = TcpStream::connect(SocketAddr::new(addr.clone(), FDROP_PORT)).await;
            if let Ok(mut sock) = stream {
                let message = definitions::protobuf::Link {
                    request: Some(true),
                    name: our_name.to_string(),
                    response: None,
                };
                let auth_message = definitions::encode(TransferType::Link, message);
                info!("sending link request to address {}", addr);
                sock.write_all(&auth_message)
                    .await
                    .map_err(|e| CommunicationError::WriteError(e))?;
                let (ttype, mut payload) = read_stream(&mut sock).await.unwrap();
                if TransferType::try_from(ttype).unwrap() != TransferType::Link {
                    error!("link request received invalid response type from peer. rejecting peer");
                }
                let resp = definitions::Link::decode(&mut payload)
                    .map_err(|_| CommunicationError::DecodeError)?;
                if matches!(
                    LinkResponse::try_from(resp.response.unwrap()).unwrap(),
                    definitions::LinkResponse::Rejected | definitions::LinkResponse::Other
                ) {
                    info!("the peer rejected the link request");
                    return Ok(LinkResponse::Rejected);
                }
                let (tx, rx) = flume::bounded(100);
                self.tx = Some(tx);
                tokio::spawn(async move {
                    handle_postauth_stream(sock, rx, handle).await;
                });
                info!("successfully linked with peer");
                return Ok(LinkResponse::Accepted);
            }
        }
        Err(CommunicationError::NoReachableAddress)
    }
}

pub struct ConnectionManager {
    mdns_daemon: ServiceDaemon,
    available_connections: HashMap<String, Connection>,
    instance_name: Option<String>,
    active_link_requests: u8,
}

impl ConnectionManager {
    pub fn new() -> Result<Mutex<Self>, NetworkError> {
        let mdns = ServiceDaemon::new().map_err(|e| DiscoveryError::ServiceDaemonError(e))?;
        mdns.set_multicast_loop_v4(false)
            .map_err(|e| DiscoveryError::ServiceDaemonError(e))?;
        mdns.set_multicast_loop_v6(false)
            .map_err(|e| DiscoveryError::ServiceDaemonError(e))?;
        Ok(Mutex::new(Self {
            mdns_daemon: mdns,
            available_connections: HashMap::new(),
            instance_name: None,
            active_link_requests: 0,
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

    pub fn get_connectionss<'a>(&'a self) -> impl Iterator<Item = &'a ConnectionInfo> {
        self.available_connections
            .values()
            .into_iter()
            .map(|c| &c.info)
    }

    pub fn get_connection_mut(&mut self, name: &str) -> Option<&mut Connection> {
        self.available_connections.get_mut(name)
    }

    pub(crate) fn connection_exists(&mut self, name: &str) -> bool {
        self.available_connections.contains_key(name)
    }
}

async fn accept_connections(handle: AppHandle) -> Result<(), CommunicationError> {
    let socket = socket2::Socket::new(Domain::IPV6, Type::STREAM, None)?;
    socket.set_only_v6(false)?;
    let address = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, FDROP_PORT, 0, 0);
    socket.bind(&address.into())?;
    socket.listen(128)?;
    let std_listener: std::net::TcpListener = socket.into();
    let listener: TcpListener = TcpListener::from_std(std_listener)?;
    info!("created the connection acceptor");

    tauri::async_runtime::spawn(async move {
        loop {
            let conn = listener.accept().await;
            match conn {
                Ok((mut stream, _)) => {
                    let handle2 = handle.clone();
                    info!("eshtablished stream with peer");
                    tauri::async_runtime::spawn(async move {
                        let ret = authenticate_peer(&mut stream, &handle2).await;
                        if let Ok(Some((rx, full_name))) = ret {
                            // HACK: Sleep for some time prevents the subsequent emit call to not hang and crash the
                            // entire app
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            handle2.emit(DEVICE_LINKED, full_name).unwrap();
                            info!("sending control of stream to post auth handler");
                            handle_postauth_stream(stream, rx, handle2).await;
                        } else {
                            info!("rejecting peer");
                        }
                    })
                    .await
                    .unwrap();
                }
                Err(e) => warn!("failed to connect to peer due to {e}"),
            }
        }
    });
    Ok(())
}

async fn launch_discovery_service(handle: AppHandle) -> Result<(), DiscoveryError> {
    let hs = whoami::fallible::hostname().map_err(|e| DiscoveryError::HostnameError(e))?;
    let local_hostname = format!("{}.local.", hs);

    let user_details_lock = handle.state::<Mutex<UserConfig>>();
    let user_details = user_details_lock.lock().await;
    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
    let mut connection_manager = cm_lock.lock().await;

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

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    if info.get_hostname() == local_hostname {
                        continue;
                    }
                    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
                    let mut connection_manager = cm_lock.lock().await;
                    let con = Connection::from(&info);
                    handle.emit(DEVICE_DISCOVERED, &con.info)?;
                    connection_manager
                        .available_connections
                        .insert(con.info.name.clone(), con);
                    info!("found device with name: {}", info.get_fullname());
                }
                ServiceEvent::ServiceRemoved(_, name) => {
                    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
                    let mut connection_manager = cm_lock.lock().await;
                    {
                        let con = connection_manager.get_connection_mut(&name).unwrap();
                        handle.emit(DEVICE_REMOVED, &con.info)?;
                        info!("'{}' left", con.info.name);
                    }
                    connection_manager.available_connections.remove(&name);
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
        Ok::<(), DiscoveryError>(())
    });
    Ok(())
}

async fn read_stream(stream: &mut TcpStream) -> Result<(TransferType, Bytes), CommunicationError> {
    let ttype_u8 = stream
        .read_u8()
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    let ttype = TransferType::try_from(ttype_u8)?;
    let payload_size = stream
        .read_u16()
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    if payload_size > (MAX_PAYLOAD_SIZE as u16) {
        // TODO: Return a response with invalid payload error
        todo!();
    }
    let mut payload = BytesMut::zeroed((payload_size).into());
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| CommunicationError::ReadError(e))?;
    Ok((ttype, payload.freeze()))
}

async fn authenticate_peer(
    stream: &mut TcpStream,
    handle: &AppHandle,
) -> Result<Option<(Receiver<Bytes>, String)>, CommunicationError> {
    info!("authenticating new peer");
    info!("reading inital message");
    let (mtype, payload) = read_stream(stream).await?;
    if mtype != TransferType::Link {
        warn!("peer sent unexpected messages before linking");
        return Err(CommunicationError::Unauthenticated);
    }

    let link_req = definitions::protobuf::Link::decode(payload);
    if link_req.is_err() {
        warn!("received invalid protobuf payload");
        return Err(CommunicationError::DecodeError);
    }
    let link_req = link_req.unwrap();

    let full_name = link_req.name.clone() + "." + MDNS_SERVICE_TYPE;
    info!(
        "received link request from peer '{}'. authenticating",
        link_req.name
    );

    let (our_name, win_label) = {
        let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        let mut connection_manager = cm_lock.lock().await;
        let our_name = connection_manager.instance_name.clone().unwrap();
        let win_label = "respond-link-request-".to_string()
            + &connection_manager.active_link_requests.to_string();
        connection_manager.active_link_requests += 1;

        if !connection_manager.connection_exists(&full_name) {
            return Err(CommunicationError::PeerNotFound);
        }
        (our_name, win_label)
    };

    let resp = confirm_link_request(handle, &link_req.name, &win_label).await;

    let ret = if resp == LinkResponse::Accepted {
        let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        let mut connection_manager = cm_lock.lock().await;
        let con = connection_manager.get_connection_mut(&full_name).unwrap();
        let (tx, rx) = bounded(100);
        con.tx = Some(tx);
        Ok(Some((rx, full_name)))
    } else {
        Ok(None)
    };

    let resp = definitions::Link {
        request: None,
        name: our_name,
        response: Some(resp as i32),
    };

    let resp_message = definitions::encode(TransferType::Link, resp);
    stream.write_all(&resp_message).await.unwrap();
    ret
}

/// Create confirmation window for a link request
#[tracing::instrument(skip(handle))]
async fn confirm_link_request(
    handle: &AppHandle,
    their_name: &str,
    win_label: &str,
) -> LinkResponse {
    info!("creating confirmation window for peer");
    let main = handle.get_webview_window("main").unwrap();
    let win = tauri::WebviewWindowBuilder::new(
        handle,
        win_label,
        WebviewUrl::App("/confirm-link-request".into()),
    )
    .title("Confirm Link Request")
    .inner_size(500.0, 200.0)
    .resizable(false)
    // Set the name of the requesting device in local storage of the window so that the
    // frontend. This is a better method than relying on tauri events which can miss if
    // they are emitted before the frontend is fully loaded.
    .initialization_script(&format!(
        "localStorage.setItem('device-name', '{}')",
        their_name
    ))
    .parent(&main)
    .unwrap()
    .build()
    .unwrap();

    let (etx, erx) = flume::bounded(1);
    win.listen(LINK_RESPONSE, move |event| match event.payload() {
        "\"accepted\"" => etx.send(LinkResponse::Accepted).unwrap(),
        "\"rejected\"" => etx.send(LinkResponse::Rejected).unwrap(),
        _ => etx.send(LinkResponse::Other).unwrap(),
    });
    let resp = erx.recv_async().await.unwrap();
    info!("user selected: {:?}", resp);
    resp
}

async fn handle_postauth_stream(mut stream: TcpStream, rx: Receiver<Bytes>, handle: AppHandle) {
    info!("issued a handler for peer");
    loop {
        tokio::select! {
            Ok(msg) = rx.recv_async() => {
                if let Err(e) = stream.write_all(&msg).await {
                    error!("failed to send message to peer: {}", e);
                }
                info!("sent message to peer")
            }
            Ok((ttype, buff)) = read_stream(&mut stream) => {
                info!("got message from peer type={:?}", ttype);
                transfer_handler(ttype, buff, &handle, &mut stream).await;
            }
        }
    }
}

async fn transfer_handler(
    ttype: TransferType,
    buff: Bytes,
    handle: &AppHandle,
    stream: &mut TcpStream,
) {
    match ttype {
        TransferType::TextMessage => {
            if let Ok(message) = definitions::protobuf::TextMessage::decode(buff) {
                let payload = Transfer {
                    ttype,
                    display_content: message.contents,
                };
                handle.emit("transfer", payload).unwrap();
            } else {
                error!("peer sent invalid bytes");
            }
        }
        TransferType::Link => {
            let user_config_lock = handle.state::<Mutex<UserConfig>>();
            let user_config = user_config_lock.lock().await;
            let our_name = user_config.instance_name.clone();

            let resp = definitions::Link {
                request: None,
                name: our_name,
                response: Some(LinkResponse::Accepted.into()),
            };

            let resp_message = definitions::encode(TransferType::Link, resp);
            stream.write_all(&resp_message).await.unwrap();
        }
    }
}

pub mod commands {
    use super::*;
    #[tauri::command]
    pub async fn enable_networking(handle: AppHandle) -> Result<(), String> {
        launch_discovery_service(handle.clone())
            .await
            .map_err(|e| NetworkError::from(e))?;
        accept_connections(handle)
            .await
            .map_err(|e| NetworkError::from(e))?;
        Ok(())
    }

    #[tauri::command]
    pub async fn send_text_message(
        handle: AppHandle,
        name: String,
        contents: String,
    ) -> Result<(), String> {
        let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        let mut connection_manager = cm_lock.lock().await;
        let con = connection_manager.get_connection_mut(&name).unwrap();

        let message = definitions::protobuf::TextMessage { contents };
        let encmsg = definitions::encode(TransferType::TextMessage, message);

        let tx = con.tx.as_mut().unwrap();
        tx.send_async(encmsg).await.unwrap();
        Ok(())
    }

    #[tauri::command]
    pub async fn link_device_by_name(
        handle: AppHandle,
        name: String,
    ) -> Result<&'static str, String> {
        let cm_lock = handle.state::<Mutex<ConnectionManager>>();
        let user_config_lock = handle.state::<Mutex<UserConfig>>();
        let mut connection_manager = cm_lock.lock().await;
        let user_config = user_config_lock.lock().await;
        let con = connection_manager.get_connection_mut(&name).unwrap();
        let our_name = &user_config.instance_name.clone();

        let res = con
            .send_link_request(handle.clone(), our_name)
            .await
            .map_err(|e| NetworkError::from(e))?;
        let res = match res {
            LinkResponse::Accepted => Ok("accepted"),
            LinkResponse::Rejected => {
                let cm_lock = handle.state::<Mutex<ConnectionManager>>();
                let mut connection_manager = cm_lock.lock().await;
                let win_label = "rejected-link-request-".to_string()
                    + &connection_manager.active_link_requests.to_string();
                connection_manager.active_link_requests += 1;

                let main = handle.get_webview_window("main").unwrap();
                tauri::WebviewWindowBuilder::new(
                    &handle,
                    win_label,
                    WebviewUrl::App("/rejected-link-request".into()),
                )
                .title("Link Request Rejected")
                .inner_size(500.0, 150.0)
                .resizable(false)
                // Set the name of the requesting device in local storage of the window so that the
                // frontend. This is a better method than relying on tauri events which can miss if
                // they are emitted before the frontend is fully loaded.
                .initialization_script(&format!(
                    "localStorage.setItem('device-name', '{}')",
                    con.info.name
                ))
                .parent(&main)
                .unwrap()
                .build()
                .unwrap();
                Ok("rejected")
            }
            LinkResponse::Other => Ok("other"),
        };
        res
    }
}
