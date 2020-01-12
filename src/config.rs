use chamomile::PeerId as PeerAddr;
use serde::de::DeserializeOwned as SeDeserializeOwned;
use serde::Serialize as SeSerialize;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;

use crate::jsonrpc::RpcConfig;
use crate::layer::LayerConfig;
use crate::p2p::P2pConfig;
use crate::primitive::{
    GroupId, CONFIG_FILE_NAME, LAYER_ADDR, LAYER_LOWER_DEFAULT, P2P_ADDR, P2P_TRANSPORT, RPC_ADDR,
};

pub struct Config {
    pub group_id: GroupId,

    pub p2p_addr: SocketAddr,
    pub p2p_join_data: Vec<u8>,
    pub p2p_transport: String,
    pub p2p_white_list: Vec<SocketAddr>,
    pub p2p_black_list: Vec<SocketAddr>,
    pub p2p_white_peer_list: Vec<PeerAddr>,
    pub p2p_black_peer_list: Vec<PeerAddr>,

    pub layer_addr: SocketAddr,
    pub layer_lower: bool,
    pub layer_upper: Vec<(SocketAddr, GroupId)>,
    pub layer_white_list: Vec<SocketAddr>,
    pub layer_black_list: Vec<SocketAddr>,
    pub layer_white_group_list: Vec<GroupId>,
    pub layer_black_group_list: Vec<GroupId>,

    pub rpc_addr: SocketAddr,
}

impl Config {
    pub fn split(self) -> (P2pConfig, LayerConfig, RpcConfig) {
        let Config {
            group_id: _,

            p2p_addr,
            p2p_join_data,
            p2p_transport,
            p2p_white_list,
            p2p_black_list,
            p2p_white_peer_list,
            p2p_black_peer_list,

            layer_addr,
            layer_lower,
            layer_upper,
            layer_white_list,
            layer_black_list,
            layer_white_group_list,
            layer_black_group_list,

            rpc_addr,
        } = self;

        let p2p_config = P2pConfig {
            addr: p2p_addr,
            join_data: p2p_join_data,
            transport: p2p_transport,
            white_list: p2p_white_list,
            black_list: p2p_black_list,
            white_peer_list: p2p_white_peer_list,
            black_peer_list: p2p_black_peer_list,
        };

        let layer_config = LayerConfig {
            addr: layer_addr,
            lower: layer_lower,
            upper: layer_upper,
            white_list: layer_white_list,
            black_list: layer_black_list,
            white_group_list: layer_white_group_list,
            black_group_list: layer_black_group_list,
        };

        let rpc_config = RpcConfig { addr: rpc_addr };

        (p2p_config, layer_config, rpc_config)
    }
}

impl Config {
    pub fn with_addr(p2p_addr: SocketAddr, layer_addr: SocketAddr, rpc_addr: SocketAddr) -> Self {
        Config {
            group_id: GroupId::default(),
            p2p_addr: p2p_addr,
            p2p_join_data: vec![],
            p2p_transport: P2P_TRANSPORT.to_owned(),
            p2p_white_list: vec![],
            p2p_black_list: vec![],
            p2p_white_peer_list: vec![],
            p2p_black_peer_list: vec![],

            layer_addr: layer_addr,
            layer_lower: LAYER_LOWER_DEFAULT,
            layer_upper: vec![],
            layer_white_list: vec![],
            layer_black_list: vec![],
            layer_white_group_list: vec![],
            layer_black_group_list: vec![],

            rpc_addr: rpc_addr,
        }
    }

    pub fn default() -> Self {
        Config::with_addr(
            P2P_ADDR.parse().unwrap(),
            LAYER_ADDR.parse().unwrap(),
            RPC_ADDR.parse().unwrap(),
        )
    }

    pub fn load() -> Self {
        let string = load_file_string();
        if string.is_none() {
            return Config::default();
        }

        let raw_config: RawConfig = toml::from_str(&string.unwrap()).unwrap();
        raw_config.parse()
    }

    pub fn load_custom<S: SeSerialize + SeDeserializeOwned>() -> Option<S> {
        let string = load_file_string();
        if string.is_none() {
            return None;
        }

        toml::from_str::<S>(&string.unwrap()).ok()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawUpper {
    addr: SocketAddr,
    group_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawConfig {
    pub group_id: Option<String>,
    pub group_symbol: Option<String>,

    pub p2p_addr: Option<SocketAddr>,
    pub p2p_join_data: Option<String>,
    pub p2p_default_transport: Option<String>,
    pub p2p_bootstrap: Vec<SocketAddr>,
    pub p2p_black_list: Option<Vec<SocketAddr>>,
    pub p2p_white_peer_list: Option<Vec<String>>,
    pub p2p_black_peer_list: Option<Vec<String>>,

    pub layer_addr: Option<SocketAddr>,
    pub layer_lower: Option<bool>,
    pub layer_upper: Option<Vec<RawUpper>>,
    pub layer_white_list: Option<Vec<SocketAddr>>,
    pub layer_black_list: Option<Vec<SocketAddr>>,
    pub layer_white_group_list: Option<Vec<String>>,
    pub layer_black_group_list: Option<Vec<String>>,

    pub rpc_addr: Option<SocketAddr>,
}

impl RawConfig {
    fn parse(self) -> Config {
        Config {
            group_id: self
                .group_id
                .map(|s| {
                    if s.len() != 64 {
                        None
                    } else {
                        let mut value = [0u8; 32];
                        let mut is_ok = true;

                        for i in 0..32 {
                            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16);
                            if res.is_err() {
                                is_ok = false;
                                break;
                            }
                            value[i] = res.unwrap()
                        }
                        if is_ok {
                            Some(GroupId(value))
                        } else {
                            None
                        }
                    }
                })
                .flatten()
                .unwrap_or(
                    self.group_symbol
                        .map(|s| GroupId::from_symbol(s))
                        .unwrap_or(GroupId::default()),
                ),
            p2p_addr: self.p2p_addr.unwrap_or(P2P_ADDR.parse().unwrap()),
            p2p_join_data: self
                .p2p_join_data
                .map(|s| {
                    let mut value: Vec<u8> = vec![];

                    for i in 0..(s.len() / 2) {
                        let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16);
                        if res.is_err() {
                            return vec![];
                        }
                        value.push(res.unwrap());
                    }
                    value
                })
                .unwrap_or(vec![]),
            p2p_transport: self
                .p2p_default_transport
                .unwrap_or(P2P_TRANSPORT.to_owned()),
            p2p_white_list: self.p2p_bootstrap,
            p2p_black_list: self.p2p_black_list.unwrap_or(vec![]),
            p2p_white_peer_list: self
                .p2p_white_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerAddr::from_hex(s).expect("invalid peer id in p2p white peer list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            p2p_black_peer_list: self
                .p2p_black_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerAddr::from_hex(s).expect("invalid group id in p2p black peer list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            layer_addr: self.layer_addr.unwrap_or(LAYER_ADDR.parse().unwrap()),
            layer_lower: self.layer_lower.unwrap_or(LAYER_LOWER_DEFAULT),
            layer_upper: self
                .layer_upper
                .map(|ss| {
                    ss.iter()
                        .map(|RawUpper { addr, group_id }| {
                            (
                                *addr,
                                GroupId::from_hex(group_id)
                                    .expect("invalid group id in layer upper"),
                            )
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            layer_white_list: self.layer_white_list.unwrap_or(vec![]),
            layer_black_list: self.layer_black_list.unwrap_or(vec![]),
            layer_white_group_list: self
                .layer_white_group_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            GroupId::from_hex(s)
                                .expect("invalid group id in layer white group list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            layer_black_group_list: self
                .layer_black_group_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            GroupId::from_hex(s)
                                .expect("invalid group id in layer black group list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),

            rpc_addr: self.rpc_addr.unwrap_or(RPC_ADDR.parse().unwrap()),
        }
    }
}

fn load_file_string() -> Option<String> {
    let mut file = match File::open(CONFIG_FILE_NAME) {
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
