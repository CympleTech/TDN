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
use std::net::SocketAddr;

use crate::group::GroupId;
use crate::{new_channel, Message};

pub struct LayerConfig {
    pub addr: SocketAddr,
    pub white_list: Vec<SocketAddr>,
    pub black_list: Vec<SocketAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>,
}

impl LayerConfig {
    pub fn default(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
            white_list: vec![],
            black_list: vec![],
            white_group_list: vec![],
            black_group_list: vec![],
        }
    }
}

pub(crate) async fn start(config: LayerConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();

    let (trans_back_send, trans_back_recv) = transport_new_channel();
    let trans_send = transport_start(&TransportType::TCP, &config.addr, trans_back_send)
        .await
        .expect("Transport binding failure!");

    // start listen self recv
    task::spawn(run_listen(send, trans_send, trans_back_recv, out_recv));

    Ok(out_send)
}

async fn run_listen(
    send: Sender<Message>,
    trans_send: Sender<EndpointMessage>,
    mut trans_recv: Receiver<EndpointMessage>,
    mut out_recv: Receiver<Message>,
) -> Result<()> {
    loop {
        select! {
            msg = trans_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from transport: {:?}", msg);
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
