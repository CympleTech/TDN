use chamomile::prelude::SendMessage;
use smol::channel::{SendError, Sender};
use tdn_types::{
    group::{GroupId, GROUP_LENGTH},
    message::{LayerReceiveMessage, LayerSendMessage, ReceiveMessage},
    primitive::{new_io_error, DeliveryType, PeerAddr, Result, StreamType},
};

#[inline]
pub(crate) async fn layer_handle_send(
    fgid: &GroupId,
    tgid: GroupId,
    p2p_send: &Sender<SendMessage>,
    msg: LayerSendMessage,
) -> std::result::Result<(), SendError<SendMessage>> {
    // fgid, tgid serialize data to msg data.
    let mut bytes = tgid.0.to_vec();
    bytes.extend(&fgid.0);
    match msg {
        LayerSendMessage::Connect(tid, peer_addr, _domain, addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableConnect(tid, peer_addr, addr, bytes))
                .await
        }
        LayerSendMessage::Disconnect(peer_addr) => {
            p2p_send
                .send(SendMessage::StableDisconnect(peer_addr))
                .await
        }
        LayerSendMessage::Result(tid, peer_addr, is_ok, is_force, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::StableResult(
                    tid, peer_addr, is_ok, is_force, bytes,
                ))
                .await
        }
        LayerSendMessage::Event(tid, peer_addr, data) => {
            bytes.extend(data);
            p2p_send
                .send(SendMessage::Data(tid, peer_addr, bytes))
                .await
        }
        LayerSendMessage::Stream(id, stream, data) => {
            bytes.extend(data);
            p2p_send.send(SendMessage::Stream(id, stream, bytes)).await
        }
    }
}

#[inline]
pub(crate) async fn layer_handle_recv_connect(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    mut data: Vec<u8>,
) -> Result<()> {
    let mut gid_bytes = [0u8; GROUP_LENGTH];
    if data.len() < GROUP_LENGTH {
        return Err(new_io_error("layer message serialize length error"));
    }
    gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
    let _tgid = GroupId(gid_bytes);
    let gmsg = LayerReceiveMessage::Connect(peer_addr, data);

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
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    is_ok: bool,
    mut data: Vec<u8>,
) -> Result<()> {
    let mut gid_bytes = [0u8; GROUP_LENGTH];
    if data.len() < GROUP_LENGTH {
        return Err(new_io_error("layer message serialize length error"));
    }
    gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
    let _tgid = GroupId(gid_bytes);
    let gmsg = LayerReceiveMessage::Result(peer_addr, is_ok, data);

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
    let gmsg = LayerReceiveMessage::Leave(peer_addr);

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
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    mut data: Vec<u8>,
) -> Result<()> {
    let mut gid_bytes = [0u8; GROUP_LENGTH];
    if data.len() < GROUP_LENGTH {
        return Err(new_io_error("layer message serialize length error"));
    }
    gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
    let _tgid = GroupId(gid_bytes);
    let gmsg = LayerReceiveMessage::Event(peer_addr, data);

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
    out_send: &Sender<ReceiveMessage>,
    uid: u32,
    stream_type: StreamType,
    mut data: Vec<u8>,
) -> Result<()> {
    let mut gid_bytes = [0u8; GROUP_LENGTH];
    if data.len() < GROUP_LENGTH {
        return Err(new_io_error("layer message serialize length error"));
    }
    gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
    let _tgid = GroupId(gid_bytes);
    let gmsg = LayerReceiveMessage::Stream(uid, stream_type, data);

    let _tgid: GroupId = Default::default();

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
    out_send: &Sender<ReceiveMessage>,
    delivery_type: DeliveryType,
    tid: u64,
    is_sended: bool,
    mut data: Vec<u8>,
) -> Result<()> {
    let mut gid_bytes = [0u8; GROUP_LENGTH];
    if data.len() < GROUP_LENGTH {
        return Err(new_io_error("layer message serialize length error"));
    }
    gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
    let _tgid = GroupId(gid_bytes);
    let gmsg = LayerReceiveMessage::Delivery(delivery_type, tid, is_sended);

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
