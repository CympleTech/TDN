use chamomile_types::message::DeliveryType as P2pDeliveryType;
use lazy_static::lazy_static;
use std::path::PathBuf;

/// P2P default binding addr.
pub const P2P_ADDR: &str = "0.0.0.0:7364";

/// P2P default transport.
pub const P2P_TRANSPORT: &str = "tcp";

/// RPC default binding addr.
pub const RPC_ADDR: &str = "127.0.0.1:8000";

/// Configure file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_STORAGE_DIR_NAME: &str = ".tdn";

lazy_static! {
    pub static ref DEFAULT_STORAGE_DIR: PathBuf = {
        #[cfg(feature = "dev")]
        let mut path = PathBuf::from("./");

        #[cfg(not(feature = "dev"))]
        let mut path = if dirs::home_dir().is_some() {
            dirs::home_dir().unwrap()
        } else {
            PathBuf::from("./")
        };

        path.push(DEFAULT_STORAGE_DIR_NAME);
        let _ = std::fs::create_dir_all(&path)
            .expect(&format!("Cannot Build Storage Path: {:?}", path));
        path
    };
}

/// Type: PeerAddr
pub type PeerAddr = chamomile_types::types::PeerId;

/// Type: P2P common Broadcast
pub use chamomile_types::types::Broadcast;

/// Type: P2P stream type.
pub use chamomile_types::message::StreamType;

/// Type: P2P transport stream type.
pub use chamomile_types::types::TransportStream;

pub type Result<T> = std::io::Result<T>;

#[inline]
pub fn new_io_error(info: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, info)
}

#[inline]
pub fn vec_remove_item<T: Eq + PartialEq>(vec: &mut Vec<T>, item: &T) {
    let mut need_remove: Vec<usize> = vec![];
    for (k, i) in vec.iter().enumerate() {
        if i == item {
            need_remove.push(k);
        }
    }

    for i in need_remove.iter().rev() {
        vec.remove(*i);
    }
}

#[inline]
pub fn vec_check_push<T: Eq + PartialEq>(vec: &mut Vec<T>, item: T) {
    for i in vec.iter() {
        if i == &item {
            return;
        }
    }

    vec.push(item);
}

/// message delivery feedback type, include three type,
/// `Connect`, `Result`, `Event`.
#[derive(Debug, Clone)]
pub enum DeliveryType {
    Event,
    Connect,
    Result,
}

impl Into<P2pDeliveryType> for DeliveryType {
    #[inline]
    fn into(self) -> P2pDeliveryType {
        match self {
            DeliveryType::Event => P2pDeliveryType::Data,
            DeliveryType::Connect => P2pDeliveryType::StableConnect,
            DeliveryType::Result => P2pDeliveryType::StableResult,
        }
    }
}

impl Into<DeliveryType> for P2pDeliveryType {
    #[inline]
    fn into(self) -> DeliveryType {
        match self {
            P2pDeliveryType::Data => DeliveryType::Event,
            P2pDeliveryType::StableConnect => DeliveryType::Connect,
            P2pDeliveryType::StableResult => DeliveryType::Result,
        }
    }
}

/// Helper: this is the group/layer/rpc handle result in the network.
pub struct HandleResult {
    /// rpc tasks: [(method, params)].
    pub rpcs: Vec<crate::rpc::RpcParam>,
    /// group tasks: [GroupSendMessage]
    pub groups: Vec<crate::message::SendType>,
    /// layer tasks: [LayerSendMessage]
    pub layers: Vec<crate::message::SendType>,
    /// network tasks: [NetworkType]
    pub networks: Vec<crate::message::NetworkType>,
}

impl<'a> HandleResult {
    pub fn new() -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![],
            networks: vec![],
        }
    }

    pub fn rpc(p: crate::rpc::RpcParam) -> Self {
        HandleResult {
            rpcs: vec![p],
            groups: vec![],
            layers: vec![],
            networks: vec![],
        }
    }

    pub fn group(m: crate::message::SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![m],
            layers: vec![],
            networks: vec![],
        }
    }

    pub fn layer(m: crate::message::SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![m],
            networks: vec![],
        }
    }

    pub fn netwrok(m: crate::message::NetworkType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![],
            networks: vec![m],
        }
    }
}
