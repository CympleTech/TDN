use chamomile_types::message::DeliveryType as P2pDeliveryType;

/// P2P default binding addr.
pub const P2P_ADDR: &str = "0.0.0.0:7364";

/// P2P default transport: QUIC.
pub const P2P_TRANSPORT: &str = "quic";

/// RPC default binding addr.
pub const RPC_ADDR: &str = "127.0.0.1:8000";

/// Configure file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_SECRET: [u8; 32] = [0u8; 32];

pub const DEFAULT_STORAGE_DIR_NAME: &str = ".tdn";

/// Type: PeerAddr
pub type PeerAddr = chamomile_types::types::PeerId;

/// Type: P2P common Broadcast
pub use chamomile_types::types::Broadcast;

/// Type: P2P stream type.
pub use chamomile_types::message::StreamType;

/// Type: P2P transport stream type.
pub use chamomile_types::types::TransportStream;

pub type Result<T> = anyhow::Result<T>;

#[inline]
pub fn new_io_error(info: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, info)
}

#[inline]
pub fn vec_remove_item<T: Eq + PartialEq>(vec: &mut Vec<T>, item: &T) {
    if let Some(pos) = vec.iter().position(|x| x == item) {
        vec.remove(pos);
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

#[cfg(not(feature = "single"))]
use crate::group::GroupId;
use crate::message::{NetworkType, SendType};
use crate::rpc::RpcParam;

/// Helper: this is the group/layer/rpc handle result in the network.
pub struct HandleResult {
    /// rpc tasks: [(method, params)].
    pub rpcs: Vec<RpcParam>,
    /// group tasks: [GroupSendMessage]
    #[cfg(any(feature = "single", feature = "std"))]
    pub groups: Vec<SendType>,
    /// group tasks: [GroupSendMessage]
    #[cfg(any(feature = "full", feature = "multiple"))]
    pub groups: Vec<(GroupId, SendType)>,
    /// layer tasks: [LayerSendMessage]
    #[cfg(feature = "std")]
    pub layers: Vec<(GroupId, SendType)>,
    /// layer tasks: [LayerSendMessage]
    #[cfg(feature = "full")]
    pub layers: Vec<(GroupId, GroupId, SendType)>,
    /// network tasks: [NetworkType]
    pub networks: Vec<NetworkType>,
}

impl<'a> HandleResult {
    pub fn new() -> Self {
        HandleResult {
            rpcs: vec![],
            #[cfg(any(
                feature = "single",
                feature = "std",
                feature = "multiple",
                feature = "full",
            ))]
            groups: vec![],
            #[cfg(any(feature = "full", feature = "std"))]
            layers: vec![],
            networks: vec![],
        }
    }

    pub fn rpc(p: RpcParam) -> Self {
        HandleResult {
            rpcs: vec![p],
            #[cfg(any(
                feature = "single",
                feature = "std",
                feature = "multiple",
                feature = "full",
            ))]
            groups: vec![],
            #[cfg(any(feature = "full", feature = "std"))]
            layers: vec![],
            networks: vec![],
        }
    }

    #[cfg(any(feature = "single", feature = "std"))]
    pub fn group(m: SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![m],
            #[cfg(feature = "std")]
            layers: vec![],
            networks: vec![],
        }
    }

    #[cfg(any(feature = "multiple", feature = "full"))]
    pub fn group(gid: GroupId, m: SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![(gid, m)],
            #[cfg(feature = "full")]
            layers: vec![],
            networks: vec![],
        }
    }

    #[cfg(feature = "std")]
    pub fn layer(gid: GroupId, m: SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![(gid, m)],
            networks: vec![],
        }
    }

    #[cfg(feature = "full")]
    pub fn layer(fgid: GroupId, tgid: GroupId, m: SendType) -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![(fgid, tgid, m)],
            networks: vec![],
        }
    }

    pub fn network(m: NetworkType) -> Self {
        HandleResult {
            rpcs: vec![],
            #[cfg(any(
                feature = "single",
                feature = "std",
                feature = "multiple",
                feature = "full",
            ))]
            groups: vec![],
            #[cfg(any(feature = "full", feature = "std"))]
            layers: vec![],
            networks: vec![m],
        }
    }
}
