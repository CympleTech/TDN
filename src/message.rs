use std::net::SocketAddr;

use crate::primitive::{Broadcast, GroupId, PeerAddr, RpcParam, StreamType};

/// channel message send to TDN Group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupSendMessage {
    /// when need add a peer, send to TDN from outside.
    /// params is `peer_id`, `socket_addr` and peer `join_info`.
    PeerConnect(PeerAddr, Option<SocketAddr>, Vec<u8>),
    /// when outside want to close a connectioned peer. use it force close.
    /// params is `peer_id`.
    PeerDisconnect(PeerAddr),
    /// when peer request for join, outside decide connect or not.
    /// params is `peer_id`, `is_connect`, `is_force_close`, `result info`.
    /// if `is_connect` is true, it will add to white directly list.
    /// we want to build a better network, add a `is_force_close`.
    /// if `is_connect` is false, but `is_force_close` if true, we
    /// will use this peer to build our DHT for better connection.
    /// if false, we will force close it.
    PeerJoinResult(PeerAddr, bool, bool, Vec<u8>),
    /// when outside want to connect a peer. will try connect directly.
    /// if connected, TDN will send PeerJoin back. if join_info is none,
    /// TDN will use config's join_data as default.
    /// params is `socket_addr`, `join_info`.
    Connect(SocketAddr, Option<Vec<u8>>),
    /// when outside donnot want to connect peer. use it to force close.
    /// params is `socket_addr`.
    DisConnect(SocketAddr),
    /// when need send a data to a peer, only need know the peer_id,
    /// the TDN will help you send data to there.
    /// params is `peer_id` and `data_bytes`.
    Data(PeerAddr, Vec<u8>),
    /// when need broadcast a data to all network, TDN support some
    /// common algorithm, use it, donnot worry.
    /// params is `broadcast_type` and `data_bytes`
    Broadcast(Broadcast, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params is `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType),
}

/// channel message receive from TDN Group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupReceiveMessage {
    /// when peer join, send from TDN to outside.
    /// params is `peer_id`, `socket_addr` and peer `join_info`.
    PeerJoin(PeerAddr, SocketAddr, Vec<u8>),
    /// when peer get join result.
    /// params is `peer_id`, `is_ok` and `result_data`.
    PeerJoinResult(PeerAddr, bool, Vec<u8>),
    /// when peer leave, send from TDN to outside.
    /// params is `peer_id`.
    PeerLeave(PeerAddr),
    /// when received a data from a trusted peer, send to outside.
    /// params is `peer_id` and `data_bytes`.
    Event(PeerAddr, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params is `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType),
}

/// channel message send to TDN Layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerSendMessage {
    /// Upper layer send to here, and return send to upper.
    Upper(GroupId, Vec<u8>),
    /// Lower layer send to here, and return send to lower.
    Lower(GroupId, Vec<u8>),
    /// start a upper layer service in layer listen. outside -> tdn.
    UpperJoin(GroupId),
    /// remove a upper layer service in layer listen. outside -> tdn.
    UpperLeave(GroupId),
    /// request for link to a upper service, and as a lower.
    /// (request_group, remote_group, uuid, addr, data).
    LowerJoin(GroupId, GroupId, u32, SocketAddr, Vec<u8>),
    /// request a upper result.
    /// (request_group, remote_group, uuid, result).
    LowerJoinResult(GroupId, GroupId, u32, bool),
}

/// channel message receive from TDN Layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerReceiveMessage {
    /// Upper layer send to here, and return send to upper.
    Upper(GroupId, Vec<u8>),
    /// Lower layer send to here, and return send to lower.
    Lower(GroupId, Vec<u8>),
    /// start a upper layer service in layer listen.
    UpperJoin(GroupId),
    /// request for link to a upper service, and as a lower.
    /// (request_group, remote_group, uuid, addr, data).
    LowerJoin(GroupId, GroupId, u32, SocketAddr, Vec<u8>),
    /// start a upper layer result. tdn -> outside
    UpperJoinResult(GroupId, bool),
    /// request a upper result.
    /// (request_group, remote_group, uuid, result).
    LowerJoinResult(GroupId, GroupId, u32, bool),
}

/// channel message send to TDN for multiple layer.
#[derive(Debug, Clone)]
pub enum SendMessage {
    /// Group: GroupMessage.
    Group(GroupSendMessage),
    /// Layer: LayerMessage.
    Layer(LayerSendMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message receive from TDN for multiple layer.
#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupReceiveMessage),
    /// Layer: LayerMessage.
    Layer(LayerReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// channel message send to TDN for signle layer.
#[derive(Debug, Clone)]
pub enum SingleSendMessage {
    /// Group: GroupMessage.
    Group(GroupSendMessage),
    /// RPC: connection uid, request params, is websocket
    Rpc(u64, RpcParam, bool),
}

/// channel message receive from TDN for signle layer.
#[derive(Debug, Clone)]
pub enum SingleReceiveMessage {
    /// Group: GroupMessage.
    Group(GroupReceiveMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// packaging the rpc message. not open to ouside.
pub(crate) struct RpcSendMessage(pub u64, pub RpcParam, pub bool);

/// generic group message for code reduce.
pub(crate) trait GroupMessage: Send {
    fn new_group(group_receive_message: GroupReceiveMessage) -> Self;
}

/// generic layer message for code reduce.
pub(crate) trait RpcMessage: Send {
    fn new_rpc(uid: u64, param: RpcParam, is_ws: bool) -> Self;
}

impl GroupMessage for ReceiveMessage {
    fn new_group(group_receive_message: GroupReceiveMessage) -> Self {
        ReceiveMessage::Group(group_receive_message)
    }
}

impl GroupMessage for SingleReceiveMessage {
    fn new_group(group_receive_message: GroupReceiveMessage) -> Self {
        SingleReceiveMessage::Group(group_receive_message)
    }
}

impl RpcMessage for ReceiveMessage {
    fn new_rpc(uid: u64, param: RpcParam, is_ws: bool) -> Self {
        ReceiveMessage::Rpc(uid, param, is_ws)
    }
}

impl RpcMessage for SingleReceiveMessage {
    fn new_rpc(uid: u64, param: RpcParam, is_ws: bool) -> Self {
        SingleReceiveMessage::Rpc(uid, param, is_ws)
    }
}
