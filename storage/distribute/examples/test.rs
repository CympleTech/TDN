use core::actor::prelude::*;
use core::crypto::hash::H256;
use core::crypto::keypair::PrivateKey;
use core::primitives::types::GroupID;
use core::{network_start, system_init, system_run};

use black_tea::{DSActor, Read, ReadResult, Write, WriteResult};

struct TestActor;

impl Actor for TestActor {
    type Context = Context<Self>;
}

impl Handler<ReadResult> for TestActor {
    type Result = ();

    fn handle(&mut self, msg: ReadResult, _ctx: &mut Self::Context) -> Self::Result {
        let (index, data) = (msg.0, msg.1);
        let s = if data.is_some() {
            String::from_utf8(data.unwrap()).unwrap()
        } else {
            String::from("None")
        };

        println!("Read Ok {}: data: {}", index, s);
    }
}

impl Handler<WriteResult> for TestActor {
    type Result = ();

    fn handle(&mut self, msg: WriteResult, _ctx: &mut Self::Context) -> Self::Result {
        let (index, id) = (msg.0, msg.1);
        let i = if id.is_some() {
            format!("{}", id.unwrap())
        } else {
            String::new()
        };

        println!("Write Ok {}: id: {}", index, i);
    }
}

fn main() {
    let runner = system_init();

    let p2p_socket = "0.0.0.0:7364".parse().unwrap();
    let rpc_socket = "0.0.0.0:3030".parse().unwrap();

    let psk = PrivateKey::from(
        "0x96bac47a4d74ddf6a2f8413c94b4c86766ef224064772d4f5afd4292d6af9e48".to_owned(),
    );

    let network_addr = network_start(p2p_socket, rpc_socket, Some(psk.clone()));

    let group_id =
        GroupID::from_str("0x0000000000000000000000000000000000000000000000000000000000000002")
            .unwrap();

    let data = "testdata1";
    let id = H256::new(data.as_bytes());

    let addr = DSActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        DSActor::load_with_psk(group_id, network_addr, psk)
    });

    let test_addr = TestActor::start(TestActor);

    addr.do_send(Write(
        test_addr.clone().recipient::<WriteResult>(),
        0u32,
        data.as_bytes().to_vec(),
    ));

    addr.do_send(Read(test_addr.clone().recipient::<ReadResult>(), 1u32, id));

    system_run(runner);
    println!("Hello, Tea!");
}
