use async_std::{
    io::Result,
    prelude::*,
    sync::{Receiver, Sender},
    task,
};
use chamomile::transports::{
    new_channel as transport_new_channel, start as transport_start, EndpointMessage, StreamMessage,
    TransportType,
};
use futures::{select, FutureExt};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::group::GroupId;
use crate::{new_channel, Message};

// if lower is ture, check black_list -> permissionless
// if lower if false, check white_list -> permissioned
pub struct LayerConfig {
    pub addr: SocketAddr,
    pub lower: bool,
    pub upper: Vec<(SocketAddr, GroupId)>,
    pub white_list: Vec<SocketAddr>,
    pub black_list: Vec<SocketAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>,
}

impl LayerConfig {
    pub fn is_close(&self) -> bool {
        self.upper.is_empty()
            && self.lower
            && self.white_list.is_empty()
            && self.white_group_list.is_empty()
    }
}

pub(crate) async fn start(config: LayerConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    if config.is_close() {
        return Ok(out_send);
    }

    let (trans_back_send, trans_back_recv) = transport_new_channel();
    let trans_send = transport_start(&TransportType::TCP, &config.addr, trans_back_send)
        .await
        .expect("Transport binding failure!");

    // start listen self recv
    task::spawn(run_listen(
        config,
        send,
        trans_send,
        trans_back_recv,
        out_recv,
    ));

    Ok(out_send)
}

async fn run_listen(
    config: LayerConfig,
    send: Sender<Message>,
    trans_send: Sender<EndpointMessage>,
    mut trans_recv: Receiver<EndpointMessage>,
    mut out_recv: Receiver<Message>,
) -> Result<()> {
    let mut uppers: HashMap<GroupId, Vec<SocketAddr>> = HashMap::new();

    // link to uppers
    for (addr, gid) in config.upper {
        uppers.insert(gid, vec![]);
        // TODO link and join bytes.
        trans_send
            .send(EndpointMessage::Connect(addr, vec![]))
            .await;
    }

    loop {
        select! {
            msg = trans_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from lower: {:?}", msg);
                    //send.send(msg).await;
                },
                None => break,
            },
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from outside: {:?}", msg);
                    //p2p_send.send(msg).await;
                },
                None => break,
            },
        }
    }

    drop(send);
    drop(trans_send);
    drop(trans_recv);
    drop(out_recv);

    Ok(())
}
