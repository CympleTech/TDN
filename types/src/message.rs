use async_channel::Sender;
use std::net::SocketAddr;

use crate::group::GroupId;
use crate::primitive::{Broadcast, DeliveryType, PeerAddr, StreamType};
use crate::rpc::RpcParam;
pub use chamomile_types::message::{StateRequest, StateResponse};

/// channel message send to TDN Group.
#[derive(Debug, Clone)]
pub enum GroupSendMessage {
    /// when need stable connect to a peer, send to TDN from outside.
    /// params: `delivery_id`, `peer_id`, `option_socket_addr` and peer `join_info`.
    StableConnect(u64, PeerAddr, Option<SocketAddr>, Vec<u8>),
    /// when outside want to close a connectioned peer. use it force close.
    /// params: `peer_id`.
    StableDisconnect(PeerAddr),
    /// when peer request for stable, outside decide connect or not.
    /// params: `delivery_id`, `peer_id`, `is_connect`, `is_force_close`, `result info`.
    /// if `is_connect` is true, it will add to allow directly list.
    /// we want to build a better network, add a `is_force_close`.
    /// if `is_connect` is false, but `is_force_close` if true, we
    /// will use this peer to build our DHT for better connection.
    /// if false, we will force close it.
    StableResult(u64, PeerAddr, bool, bool, Vec<u8>),
    /// when need send a data to a peer, only need know the peer_id,
    /// the TDN will help you send data to there.
    /// params: `delivery_id`, `peer_id`, `data_bytes`.
    Event(u64, PeerAddr, Vec<u8>),
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
    /// Apply for build a stream between nodes.
    /// params: `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType, Vec<u8>),
    /// Request for return the network current state info.
    /// params: request type, and return channel's sender (async).
    NetworkState(StateRequest, Sender<StateResponse>),
}

/// channel message receive from TDN Group.
#[derive(Debug, Clone)]
pub enum GroupReceiveMessage {
    /// when peer what a stable connection, send from TDN to outside.
    /// params: `peer_id`, and peer `connect_info`.
    StableConnect(PeerAddr, Vec<u8>),
    /// when peer a stable connect result.
    /// params: `peer_id`, `is_ok` and `result_data`.
    StableResult(PeerAddr, bool, Vec<u8>),
    /// when a stable connected peer leave, send from TDN to outside.
    /// params: `peer_id`.
    StableLeave(PeerAddr),
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

/// channel message send to TDN Layers.
#[derive(Debug, Clone)]
pub enum LayerSendMessage {
    /// layer connect to other Group. it will be stable connection.
    /// params: `delivery_id`, `peer_id`, `option_domain`, `option_socket`, `data`.
    Connect(u64, PeerAddr, Option<String>, Option<SocketAddr>, Vec<u8>),
    /// when outside want to close a connectioned peer. use it force close.
    /// params: `peer_id`.
    Disconnect(PeerAddr),
    /// layer connect result. it will be stable connection.
    /// params: `delivery_id`, `peer_id`, `is_ok`, `is_force_close`, `data`.
    Result(u64, PeerAddr, bool, bool, Vec<u8>),
    /// layer send message.
    /// params: `delivery_id`, `peer_id`, `data`.
    Event(u64, PeerAddr, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params: `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType, Vec<u8>),
}

/// channel message receive from TDN Layers.
#[derive(Debug, Clone)]
pub enum LayerReceiveMessage {
    /// layer connect to other Group. it will be stable connection.
    /// params: `peer_id`, `data`.
    Connect(PeerAddr, Vec<u8>),
    /// layer connect result. it will be stable connection.
    /// params: `is_ok`, `data`.
    Result(PeerAddr, bool, Vec<u8>),
    /// layer connection leave. when peer leave, you need handle it by yourself.
    /// it will make from_group_id, to_group_id all set my_group_id.
    Leave(PeerAddr),
    /// layer send message.
    /// params: `data`.
    Event(PeerAddr, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params: `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType, Vec<u8>),
    /// Message sended delivery feedback. type has Event, Connect, Result.
    /// params: `delivery_type`, `delivery_id`, `is_sended`.
    Delivery(DeliveryType, u64, bool),
}

/// channel message send to TDN for std version.
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupSendMessage),
    /// Layer: LayerMessage.
    Layer(GroupId, LayerSendMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message receive from TDN for std version.
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupReceiveMessage),
    /// Layer: LayerMessage. Take care of `Leave`.
    Layer(GroupId, LayerReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message send to TDN for single version.
#[cfg(feature = "single")]
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupSendMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message receive from TDN for single version.
#[cfg(feature = "single")]
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message send to TDN for multiple version.
#[cfg(feature = "multiple")]
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupId, GroupSendMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(GroupId, u64, RpcParam, bool),
}

/// channel message receive from TDN for multiple version.
#[cfg(feature = "multiple")]
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupId, GroupReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(GroupId, u64, RpcParam, bool),
}

/// channel message send to TDN for full version.
#[cfg(feature = "full")]
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupId, GroupSendMessage),
    /// Layer: LayerMessage.
    /// params: sender's id, receiver's, msg. Take care of `Leave`.
    Layer(GroupId, GroupId, LayerSendMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(GroupId, u64, RpcParam, bool),
}

/// channel message receive from TDN for full version.
#[cfg(feature = "full")]
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupId, GroupReceiveMessage),
    /// Layer: LayerMessage.
    /// params: receiver's id, sender's, msg.
    Layer(GroupId, GroupId, LayerReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(GroupId, u64, RpcParam, bool),
}

/// packaging the rpc message. not open to ouside.
pub struct RpcSendMessage(pub u64, pub RpcParam, pub bool);

/// generic layer message for code reduce.
pub trait RpcMessage: Send {
    fn new_rpc(uid: u64, param: RpcParam, is_ws: bool) -> Self;
}

impl RpcMessage for ReceiveMessage {
    fn new_rpc(uid: u64, param: RpcParam, is_ws: bool) -> Self {
        ReceiveMessage::Rpc(uid, param, is_ws)
    }
}
