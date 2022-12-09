use chamomile_types::message::DeliveryType as P2pDeliveryType;
use chamomile_types::Peer as ChamomilePeer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Type: PeerId, PeerKey
pub use chamomile_types::{
    key::{secp256k1, Key as PeerKey, PublicKey as PeerPublicKey, SecretKey as PeerSecretKey},
    types::{PeerId, TransportType, PEER_ID_LENGTH},
};

/// Type: P2P common Broadcast
pub use chamomile_types::types::Broadcast;

/// Type: P2P stream type.
pub use chamomile_types::message::StreamType;

/// Type: P2P transport stream type.
pub use chamomile_types::types::TransportStream;

/// P2P default binding addr.
pub const P2P_ADDR: &str = "0.0.0.0:7364";

/// P2P default transport: QUIC.
pub const P2P_TRANSPORT: TransportType = TransportType::QUIC;

/// RPC default binding addr.
pub const RPC_HTTP: &str = "127.0.0.1:7365";

/// Configure file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_SECRET: [u8; 32] = [0u8; 32];

pub const DEFAULT_STORAGE_DIR_NAME: &str = ".tdn";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Peer {
    pub id: PeerId,
    pub socket: SocketAddr,
    pub transport: TransportType,
    pub httpurl: String,
    pub is_pub: bool,
}

impl Peer {
    pub fn socket(socket: SocketAddr) -> Peer {
        Self {
            id: PeerId::default(),
            socket: socket,
            transport: P2P_TRANSPORT,
            httpurl: String::new(),
            is_pub: true,
        }
    }

    pub fn socket_transport(socket: SocketAddr, trans: &str) -> Peer {
        Self {
            id: PeerId::default(),
            socket: socket,
            transport: TransportType::from_str(trans),
            httpurl: String::new(),
            is_pub: true,
        }
    }

    pub fn peer(peer_id: PeerId) -> Peer {
        Self {
            id: peer_id,
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            transport: P2P_TRANSPORT,
            httpurl: String::new(),
            is_pub: true,
        }
    }

    /// Enhanced multiaddr, you can import/export it.
    /// example: "p2p::xxx::/ip4/127.0.0.1/tcp/1234"
    /// example: "rpc::xxx::http://example.com"
    pub fn to_string(&self) -> String {
        if self.httpurl.len() > 0 {
            format!("rpc::{}::{}", self.id.to_hex(), self.httpurl)
        } else {
            let p2p = ChamomilePeer {
                id: self.id,
                socket: self.socket,
                transport: self.transport,
                is_pub: self.is_pub,
                assist: self.id,
            };
            format!("p2p::{}::{}", self.id.to_hex(), p2p.to_multiaddr_string())
        }
    }

    pub fn from_string(s: &str) -> Result<Peer> {
        let mut ss = s.split("::");

        let protocol = ss.next().ok_or(new_io_error("peer string is invalid."))?;
        let id = PeerId::from_hex(ss.next().ok_or(new_io_error("peer string is invalid."))?)?;

        if protocol == "rpc" {
            let mut peer = Peer::default();
            peer.id = id;
            peer.httpurl = ss
                .next()
                .ok_or(new_io_error("peer string is invalid."))?
                .to_owned();
            Ok(peer)
        } else {
            let p2p = ChamomilePeer::from_multiaddr_string(
                ss.next().ok_or(new_io_error("peer string is invalid."))?,
            )?;
            let mut peer = Peer::from(p2p);
            peer.id = id;
            Ok(peer)
        }
    }
}

impl Default for Peer {
    fn default() -> Peer {
        Peer {
            id: PeerId::default(),
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            transport: P2P_TRANSPORT,
            httpurl: String::new(),
            is_pub: false,
        }
    }
}

impl From<ChamomilePeer> for Peer {
    fn from(cp: ChamomilePeer) -> Peer {
        Peer {
            id: cp.id,
            socket: cp.socket,
            transport: cp.transport,
            is_pub: cp.is_pub,
            httpurl: String::new(),
        }
    }
}

impl Into<ChamomilePeer> for Peer {
    fn into(self) -> ChamomilePeer {
        ChamomilePeer {
            id: self.id,
            socket: self.socket,
            transport: self.transport,
            is_pub: self.is_pub,
            assist: self.id,
        }
    }
}

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
    /// P2P network with same PeerId.
    pub owns: Vec<SendType>,
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
            owns: vec![],
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

    pub fn own(m: SendType) -> Self {
        HandleResult {
            owns: vec![m],
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
            owns: vec![],
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
            owns: vec![],
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
            owns: vec![],
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
            owns: vec![],
            rpcs: vec![],
            groups: vec![],
            layers: vec![(gid, m)],
            networks: vec![],
        }
    }

    #[cfg(feature = "full")]
    pub fn layer(fgid: GroupId, tgid: GroupId, m: SendType) -> Self {
        HandleResult {
            owns: vec![],
            rpcs: vec![],
            groups: vec![],
            layers: vec![(fgid, tgid, m)],
            networks: vec![],
        }
    }

    pub fn network(m: NetworkType) -> Self {
        HandleResult {
            owns: vec![],
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
