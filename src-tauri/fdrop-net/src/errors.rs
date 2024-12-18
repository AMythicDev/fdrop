use fdrop_common::human_readable_error;
#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    #[error("discovery error")]
    DiscoveryError(#[from] DiscoveryError),
    #[error("failed to communicate on the network")]
    CommunicationError(#[from] CommunicationError),
}

#[derive(thiserror::Error, Debug)]
pub enum CommunicationError {
    #[error("failed to write to the socket")]
    WriteError(std::io::Error),
    #[error("failed to read to the socket")]
    ReadError(std::io::Error),
    #[error("failed to decode peer message")]
    DecodeError,
    #[error("no reachable address for the peer")]
    NoReachableAddress,
    #[error("peer sent unexpected messages before linking")]
    Unauthenticated,
    #[error("peer not found by discovery service")]
    PeerNotFound,
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

impl From<NetworkError> for String {
    fn from(value: NetworkError) -> Self {
        human_readable_error(&value)
    }
}
