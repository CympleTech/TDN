use serde::de::DeserializeOwned as SeDeserializeOwned;
use serde::Serialize as SeSerialize;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, SocketAddr};

use crate::primitives::consts::{P2P_DEFAULT_SOCKET, RPC_DEFAULT_SOCKET};
use crate::primitives::types::{GroupID, PeerAddr as NodeAddr};

#[derive(Serialize, Deserialize, Debug)]
struct Socket {
    ip: IpAddr,
    port: u16,
}

impl Socket {
    fn parse(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PeerAddr {
    ip: IpAddr,
    port: u16,
    pk: String,
}

impl PeerAddr {
    fn parse(&self) -> (NodeAddr, SocketAddr) {
        ((&self.pk).into(), SocketAddr::new(self.ip, self.port))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigureRow {
    current_group: String,
    upper_group: String,
    p2p_address: Socket,
    rpc_address: Socket,
    upper_address: Socket,
    lower_address: Socket,
    bootstrap_peers: Vec<PeerAddr>,
}

impl ConfigureRow {
    fn parse(&self) -> Configure {
        let current_group = GroupID::from_string(&self.current_group).unwrap();
        let upper_group = GroupID::from_string(&self.upper_group).unwrap();
        let p2p_socket = self.p2p_address.parse();
        let rpc_socket = self.rpc_address.parse();
        let upper_address = self.upper_address.parse();
        let lower_address = self.lower_address.parse();
        let bootstrap_peers = self.bootstrap_peers.iter().map(|p| p.parse()).collect();

        Configure::new(
            current_group,
            upper_group,
            p2p_socket,
            rpc_socket,
            upper_address,
            lower_address,
            bootstrap_peers,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configure {
    pub current_group: GroupID,
    pub upper_group: GroupID,
    pub p2p_address: SocketAddr,
    pub rpc_address: SocketAddr,
    pub upper_address: SocketAddr,
    pub lower_address: SocketAddr,
    pub bootstrap_peers: Vec<(NodeAddr, SocketAddr)>,
}

impl Configure {
    fn new(
        current_group: GroupID,
        upper_group: GroupID,
        p2p_address: SocketAddr,
        rpc_address: SocketAddr,
        upper_address: SocketAddr,
        lower_address: SocketAddr,
        bootstrap_peers: Vec<(NodeAddr, SocketAddr)>,
    ) -> Self {
        Configure {
            current_group,
            upper_group,
            p2p_address,
            rpc_address,
            upper_address,
            lower_address,
            bootstrap_peers,
        }
    }

    fn default() -> Self {
        let p2p_address = P2P_DEFAULT_SOCKET.parse().unwrap();
        let rpc_address = RPC_DEFAULT_SOCKET.parse().unwrap();
        let upper_address = RPC_DEFAULT_SOCKET.parse().unwrap();
        let lower_address = RPC_DEFAULT_SOCKET.parse().unwrap();
        let current_group = GroupID::from_str("0x00000000000000000000000000000000").unwrap();
        let upper_group = GroupID::from_str("0x00000000000000000000000000000000").unwrap();
        let bootstrap_peers = vec![];

        Configure {
            current_group,
            upper_group,
            p2p_address,
            rpc_address,
            upper_address,
            lower_address,
            bootstrap_peers,
        }
    }

    pub fn load(_pk: Option<&NodeAddr>) -> Self {
        let string = load_file_string();
        if string.is_none() {
            return Configure::default();
        }

        let config_row: ConfigureRow = toml::from_str(&string.unwrap()).unwrap();
        config_row.parse()
    }

    pub fn load_custom<S: SeSerialize + SeDeserializeOwned>() -> Option<S> {
        let string = load_file_string();
        if string.is_none() {
            return None;
        }

        toml::from_str::<S>(&string.unwrap()).ok()
    }
}

fn load_file_string() -> Option<String> {
    let file_path = "config.toml";
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => {
            return None;
        }
    };

    let mut str_val = String::new();
    match file.read_to_string(&mut str_val) {
        Ok(s) => s,
        Err(e) => panic!("Error Reading file: {}", e),
    };
    Some(str_val)
}
