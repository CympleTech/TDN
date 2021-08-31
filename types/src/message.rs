use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

use crate::primitive::{Broadcast, DeliveryType, PeerAddr, StreamType};
use crate::rpc::RpcParam;
pub use chamomile_types::message::{StateRequest, StateResponse};

#[cfg(not(feature = "single"))]
use crate::group::GroupId;

/// channel message send to TDN Group.
#[derive(Debug)]
pub enum SendType {
    /// when need stable connect to a peer, send to TDN from outside.
    /// params: `delivery_id`, `peer_id`, `option_domain`, `option_socket_addr` and peer `join_data`.
    Connect(u64, PeerAddr, Option<String>, Option<SocketAddr>, Vec<u8>),
    /// when peer request for stable, outside decide connect or not.
    /// params: `delivery_id`, `peer_id`, `is_connect`, `is_force_close`, `result_data`.
    /// if `is_connect` is true, it will add to allow directly list.
    /// we want to build a better network, add a `is_force_close`.
    /// if `is_connect` is false, but `is_force_close` if true, we
    /// will use this peer to build our DHT for better connection.
    /// if false, we will force close it.
    Result(u64, PeerAddr, bool, bool, Vec<u8>),
    /// when outside want to close a connectioned peer. use it force close.
    /// params: `peer_id`.
    Disconnect(PeerAddr),
    /// when need send a data to a peer, only need know the peer_id,
    /// the TDN will help you send data to there.
    /// params: `delivery_id`, `peer_id`, `data_bytes`.
    Event(u64, PeerAddr, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params: `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType, Vec<u8>),
}

/// channel message receive from TDN Group.
#[derive(Debug)]
pub enum RecvType {
    /// when peer what a stable connection, send from TDN to outside.
    /// params: `peer_id`, and peer `connect_info`.
    Connect(PeerAddr, Vec<u8>),
    /// when peer a stable connect result.
    /// params: `peer_id`, `is_ok` and `result_data`.
    Result(PeerAddr, bool, Vec<u8>),
    /// when peer agree a connect, but network is closed,
    /// create a result connect to it.
    ResultConnect(PeerAddr, Vec<u8>),
    /// when a stable connected peer leave, send from TDN to outside.
    /// params: `peer_id`.
    Leave(PeerAddr),
    /// when received a data from a trusted peer, send to outside.
    /// params: `peer_id` and `data_bytes`.
    Event(PeerAddr, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params: `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType, Vec<u8>),
    /// Message sended delivery feedback. type has Event, StableConnect, StableResult.
    /// params: `delivery_type`, `delivery_id`, `is_sended`.
    Delivery(DeliveryType, u64, bool),
}

/// channel message send to chamomile network.
#[derive(Debug, Clone)]
pub enum NetworkType {
    /// when outside want to add a peer to bootstrap and DHT.
    /// if connected, TDN will add it to boostrap and DHT.
    /// params: `socket_addr`.
    Connect(SocketAddr),
    /// when outside donnot want to remove peer. use it to force close.
    /// params: `socket_addr`.
    DisConnect(SocketAddr),
    /// when need broadcast a data to all network, TDN support some
    /// common algorithm, use it, donnot worry.
    /// params: `broadcast_type` and `data_bytes`
    Broadcast(Broadcast, Vec<u8>),
    /// Request for return the network current state info.
    /// params: request type, and return channel's sender (async).
    NetworkState(StateRequest, Sender<StateResponse>),
    /// When receive `ReceiveMessage::NetworkLost`, want to reboot network, it can use.
    NetworkReboot,
    /// add group to TDN control. multiple group use.
    #[cfg(any(feature = "multiple", feature = "full"))]
    AddGroup(GroupId),
    /// remove group from TDN control. multiple group use.
    #[cfg(any(feature = "multiple", feature = "full"))]
    DelGroup(GroupId),
}

/// channel message send to TDN for std version.
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(SendType),
    /// Layer: LayerMessage.
    Layer(GroupId, SendType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// Network: Control the Network state.
    Network(NetworkType),
}

/// channel message receive from TDN for std version.
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(RecvType),
    /// Layer: LayerMessage. Take care of `Leave`.
    Layer(GroupId, RecvType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// when network lost all DHT network and direct stables. will tell outside.
    NetworkLost,
}

/// channel message send to TDN for single version.
#[cfg(feature = "single")]
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(SendType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// Network: Control the Network state.
    Network(NetworkType),
}

/// channel message receive from TDN for single version.
#[cfg(feature = "single")]
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(RecvType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// when network lost all DHT network and direct stables. will tell outside.
    NetworkLost,
}

/// channel message send to TDN for multiple version.
#[cfg(feature = "multiple")]
#[derive(Debug)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupId, SendType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// Network: Control the Network state.
    Network(NetworkType),
}

/// channel message receive from TDN for multiple version.
#[cfg(feature = "multiple")]
#[derive(Debug)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupId, RecvType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// when network lost all DHT network and direct stables. will tell outside.
    NetworkLost,
}

/// channel message send to TDN for full version.
#[cfg(feature = "full")]
#[derive(Debug)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupId, SendType),
    /// Layer: LayerMessage.
    /// params: sender's id, receiver's, msg. Take care of `Leave`.
    Layer(GroupId, GroupId, SendType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// Network: Control the Network state.
    Network(NetworkType),
}

/// channel message receive from TDN for full version.
#[cfg(feature = "full")]
#[derive(Debug)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupId, RecvType),
    /// Layer: LayerMessage.
    /// params: sender's, receiver's, msg.
    Layer(GroupId, GroupId, RecvType),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
    /// when network lost all DHT network and direct stables. will tell outside.
    NetworkLost,
}

/// packaging the rpc message. not open to ouside.
#[derive(Debug)]
pub struct RpcSendMessage(pub u64, pub RpcParam, pub bool);
