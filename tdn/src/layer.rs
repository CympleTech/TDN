use chamomile::prelude::SendMessage;
use tdn_types::{
    group::GroupId,
    message::{ReceiveMessage, RecvType, SendType},
    primitives::{DeliveryType, Peer, PeerId, Result, StreamType},
};
use tokio::sync::mpsc::{error::SendError, Sender};

#[inline]
pub(crate) async fn layer_handle_send(
    fgid: GroupId,
    tgid: GroupId,
    p2p_send: &Sender<SendMessage>,
    msg: SendType,
) -> std::result::Result<(), SendError<SendMessage>> {
    // fgid, tgid serialize data to msg data.
    let mut bytes = fgid.to_be_bytes().to_vec();
    bytes.extend(&tgid.to_be_bytes());
    match msg {
        SendType::Connect(tid, peer, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableConnect(tid, peer.into(), bytes))
                .await
        }
        SendType::Disconnect(peer_id) => {
            p2p_send.send(SendMessage::StableDisconnect(peer_id)).await
        }
        SendType::Result(tid, peer, is_ok, is_force, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableResult(
                    tid,
                    peer.into(),
                    is_ok,
                    is_force,
                    bytes,
                ))
                .await
        }
        SendType::Event(tid, peer_id, data) => {
            bytes.extend(data);
            p2p_send.send(SendMessage::Data(tid, peer_id, bytes)).await
        }
        SendType::Stream(id, stream, data) => {
            bytes.extend(data);
            p2p_send.send(SendMessage::Stream(id, stream, bytes)).await
        }
    }
}

#[inline]
pub(crate) async fn layer_handle_connect(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer: Peer,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Connect(peer, data);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_result_connect(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer: Peer,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::ResultConnect(peer, data);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_result(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer: Peer,
    is_ok: bool,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Result(peer, is_ok, data);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_leave(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer: impl Into<Peer>,
) -> Result<()> {
    let gmsg = RecvType::Leave(peer.into());
    let msg = ReceiveMessage::Layer(fgid, fgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_data(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_id: PeerId,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Event(peer_id, data);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_stream(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    uid: u32,
    stream_type: StreamType,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = RecvType::Stream(uid, stream_type, data);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_delivery(
    fgid: GroupId,
    tgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    delivery_type: DeliveryType,
    tid: u64,
    is_sended: bool,
) -> Result<()> {
    let gmsg = RecvType::Delivery(delivery_type, tid, is_sended);
    let msg = ReceiveMessage::Layer(fgid, tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e));

    Ok(())
}
