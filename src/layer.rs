use chamomile::prelude::SendMessage;
use smol::channel::{SendError, Sender};
use tdn_types::{
    group::GroupId,
    message::{ReceiveMessage, RecvType, SendType},
    primitive::{DeliveryType, PeerAddr, Result, StreamType},
};

#[inline]
pub(crate) async fn layer_handle_send(
    fgid: GroupId,
    tgid: GroupId,
    p2p_send: &Sender<SendMessage>,
    msg: SendType,
) -> std::result::Result<(), SendError<SendMessage>> {
    // fgid, tgid serialize data to msg data.
    let mut bytes = fgid.0.to_vec();
    bytes.extend(&tgid.0);
    match msg {
        SendType::Connect(tid, peer_addr, _domain, addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableConnect(tid, peer_addr, addr, bytes))
                .await
        }
        SendType::Disconnect(peer_addr) => {
            p2p_send
                .send(SendMessage::StableDisconnect(peer_addr))
                .await
        }
        SendType::Result(tid, peer_addr, is_ok, is_force, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableResult(
                    tid, peer_addr, is_ok, is_force, bytes,
                ))
                .await
        }
        SendType::Event(tid, peer_addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::Data(tid, peer_addr, bytes))
                .await
        }
        SendType::Stream(id, stream, data) => {
            bytes.extend(data);
            p2p_send.send(SendMessage::Stream(id, stream, bytes)).await
        }
    }
}

#[inline]
pub(crate) async fn layer_handle_recv_connect(
    fgid: GroupId,
    _tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Connect(peer_addr, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_result(
    fgid: GroupId,
    _tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    is_ok: bool,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Result(peer_addr, is_ok, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_leave(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
) -> Result<()> {
    let gmsg = RecvType::Leave(peer_addr);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, fgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_data(
    fgid: GroupId,
    _tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Event(peer_addr, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_stream(
    fgid: GroupId,
    _tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    uid: u32,
    stream_type: StreamType,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Stream(uid, stream_type, data);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_delivery(
    fgid: GroupId,
    _tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    delivery_type: DeliveryType,
    tid: u64,
    is_sended: bool,
) -> Result<()> {
    let gmsg = RecvType::Delivery(delivery_type, tid, is_sended);

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}
