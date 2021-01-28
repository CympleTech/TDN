use chamomile::prelude::SendMessage;
use smol::channel::{SendError, Sender};
use tdn_types::{
    group::GroupId,
    message::{GroupReceiveMessage, GroupSendMessage, ReceiveMessage},
    primitive::{DeliveryType, PeerAddr, Result, StreamType},
};

#[inline]
pub(crate) async fn group_handle_send(
    gid: &GroupId,
    p2p_send: &Sender<SendMessage>,
    msg: GroupSendMessage,
) -> std::result::Result<(), SendError<SendMessage>> {
    // gid serialize data to msg data.
    let mut bytes = gid.0.to_vec();
    match msg {
        GroupSendMessage::StableConnect(tid, peer_addr, addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableConnect(tid, peer_addr, addr, bytes))
                .await
        }
        GroupSendMessage::StableDisconnect(peer_addr) => {
            p2p_send
                .send(SendMessage::StableDisconnect(peer_addr))
                .await
        }
        GroupSendMessage::StableResult(tid, peer_addr, is_ok, is_force, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableResult(
                    tid, peer_addr, is_ok, is_force, bytes,
                ))
                .await
        }
        GroupSendMessage::Event(tid, peer_addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::Data(tid, peer_addr, bytes))
                .await
        }
        GroupSendMessage::Broadcast(broadcast, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::Broadcast(broadcast, bytes))
                .await
        }
        GroupSendMessage::Stream(id, stream, data) => {
            bytes.extend(data);
            p2p_send.send(SendMessage::Stream(id, stream, bytes)).await
        }
        GroupSendMessage::Connect(addr) => p2p_send.send(SendMessage::Connect(addr)).await,
        GroupSendMessage::DisConnect(addr) => p2p_send.send(SendMessage::DisConnect(addr)).await,
        GroupSendMessage::NetworkState(req, sender) => {
            p2p_send.send(SendMessage::NetworkState(req, sender)).await
        }
    }
}

#[inline]
pub(crate) async fn group_handle_recv_stable_connect(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::StableConnect(peer_addr, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn group_handle_recv_stable_result(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    is_ok: bool,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::StableResult(peer_addr, is_ok, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn group_handle_recv_stable_leave(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::StableLeave(peer_addr);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn group_handle_recv_data(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::Event(peer_addr, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn group_handle_recv_stream(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    uid: u32,
    stream_type: StreamType,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::Stream(uid, stream_type, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn group_handle_recv_delivery(
    _gid: &GroupId,
    out_send: &Sender<ReceiveMessage>,
    delivery_type: DeliveryType,
    tid: u64,
    is_sended: bool,
) -> Result<()> {
    let gmsg = GroupReceiveMessage::Delivery(delivery_type, tid, is_sended);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Group(gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Group(*_gid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}
