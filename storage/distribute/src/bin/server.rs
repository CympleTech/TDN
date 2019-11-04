use std::net::SocketAddr;

use core::actor::prelude::Actor;
use core::crypto::keypair::PrivateKey;
use core::primitives::types::GroupID;
use core::Configure;
use core::{network_start, system_init, system_run};

use black_tea::DSActor;

fn main() {
    let runner = system_init();

    let configure = Configure::load(None);
    let group_id: GroupID = configure.current_group;
    let p2p_socket: SocketAddr = configure.p2p_address;
    let rpc_socket: SocketAddr = configure.rpc_address;

    //let psk = PrivateKey::from(
    //"0x96bac47a4d74ddf6a2f8413c94b4c86766ef224064772d4f5afd4292d6af9e48".to_owned(),
    //);
    let psk = PrivateKey::generate();

    let network_addr = network_start(p2p_socket, rpc_socket, Some(psk.clone()));

    DSActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        DSActor::load_with_psk(group_id, network_addr, psk)
    });

    system_run(runner);
    println!("Hello, Tea!");
}
