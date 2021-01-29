use serde::de::DeserializeOwned as SeDeserializeOwned;
use serde::ser::Serialize as SeSerialize;
use serde::{Deserialize, Serialize};
use smol::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use chamomile::prelude::Config as P2pConfig;
use tdn_types::{
    group::{GroupId, GROUP_LENGTH},
    primitive::{
        PeerAddr, Result, CONFIG_FILE_NAME, DEFAULT_STORAGE_DIR, P2P_ADDR, P2P_TRANSPORT, RPC_ADDR,
    },
};

use crate::rpc::RpcConfig;

/// load config from config file.
pub struct Config {
    pub db_path: Option<PathBuf>,
    pub group_ids: Vec<GroupId>,
    pub permission: bool,
    pub only_stable_data: bool,

    pub p2p_addr: SocketAddr,
    pub p2p_transport: String,
    pub p2p_allowlist: Vec<SocketAddr>,
    pub p2p_blocklist: Vec<IpAddr>,
    pub p2p_allow_peer_list: Vec<PeerAddr>,
    pub p2p_block_peer_list: Vec<PeerAddr>,

    pub rpc_addr: SocketAddr,
    pub rpc_ws: Option<SocketAddr>,
    pub rpc_index: Option<PathBuf>,
}

impl Config {
    pub fn split(self) -> (Vec<GroupId>, P2pConfig, RpcConfig) {
        #[cfg(feature = "single")]
        let delivery_length = 0;
        #[cfg(feature = "multiple")]
        let delivery_length = GROUP_LENGTH;
        #[cfg(any(feature = "std", feature = "full"))]
        let delivery_length = GROUP_LENGTH * 2;

        let Config {
            db_path,
            group_ids,

            permission,
            only_stable_data,
            p2p_addr,
            p2p_transport,
            p2p_allowlist,
            p2p_blocklist,
            p2p_allow_peer_list,
            p2p_block_peer_list,

            rpc_addr,
            rpc_ws,
            rpc_index,
        } = self;

        let p2p_config = P2pConfig {
            db_dir: if let Some(path) = db_path {
                path
            } else {
                DEFAULT_STORAGE_DIR.clone()
            },
            addr: p2p_addr,
            transport: p2p_transport,
            allowlist: p2p_allowlist,
            blocklist: p2p_blocklist,
            allow_peer_list: p2p_allow_peer_list,
            block_peer_list: p2p_block_peer_list,
            permission: permission,
            only_stable_data: only_stable_data,
            delivery_length: delivery_length,
        };

        let rpc_config = RpcConfig {
            addr: rpc_addr,
            ws: rpc_ws,
            index: rpc_index,
        };

        (group_ids, p2p_config, rpc_config)
    }
}

impl Config {
    pub fn with_addr(p2p_addr: SocketAddr, rpc_addr: SocketAddr) -> Self {
        Config {
            db_path: None,
            group_ids: vec![GroupId::default()],
            permission: false,       // default is permissionless
            only_stable_data: false, // default is permissionless
            p2p_addr: p2p_addr,
            p2p_transport: P2P_TRANSPORT.to_owned(),
            p2p_allowlist: vec![],
            p2p_blocklist: vec![],
            p2p_allow_peer_list: vec![],
            p2p_block_peer_list: vec![],

            rpc_addr: rpc_addr,
            rpc_ws: None,
            rpc_index: None,
        }
    }

    pub fn default() -> Self {
        Config::with_addr(P2P_ADDR.parse().unwrap(), RPC_ADDR.parse().unwrap())
    }

    pub async fn load() -> Self {
        let string = load_file_string(DEFAULT_STORAGE_DIR.clone()).await;

        match string {
            Ok(string) => {
                let raw_config: RawConfig = toml::from_str(&string).unwrap();
                raw_config.parse()
            }
            Err(_) => Config::default(),
        }
    }

    pub async fn load_with_path(path: PathBuf) -> Self {
        let string = load_file_string(path.clone()).await;
        match string {
            Ok(string) => {
                let raw_config: RawConfig = toml::from_str(&string).unwrap();
                raw_config.parse()
            }
            Err(_err) => {
                warn!("File {:?} not found, use the config default.", path);
                Config::default()
            }
        }
    }

    pub async fn load_custom<S: SeSerialize + SeDeserializeOwned>() -> Option<S> {
        let string = load_file_string(DEFAULT_STORAGE_DIR.clone()).await;
        match string {
            Ok(string) => toml::from_str::<S>(&string).ok(),
            Err(_) => None,
        }
    }

    pub async fn load_custom_with_path<S: SeSerialize + SeDeserializeOwned>(
        path: PathBuf,
    ) -> Option<S> {
        let string = load_file_string(path).await;
        match string {
            Ok(string) => toml::from_str::<S>(&string).ok(),
            Err(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawUpper {
    addr: SocketAddr,
    group_id: String,
}

/// parse raw file content to Config.
#[derive(Serialize, Deserialize, Debug)]
pub struct RawConfig {
    pub db_path: Option<PathBuf>,
    pub group_id: Option<String>,
    pub group_symbol: Option<String>,
    pub permission: Option<bool>,
    pub only_stable_data: Option<bool>,

    pub p2p_addr: Option<SocketAddr>,
    pub p2p_default_transport: Option<String>,
    pub p2p_bootstrap: Vec<SocketAddr>,
    pub p2p_blocklist: Option<Vec<IpAddr>>,
    pub p2p_allow_peer_list: Option<Vec<String>>,
    pub p2p_block_peer_list: Option<Vec<String>>,

    pub rpc_addr: Option<SocketAddr>,
    pub rpc_ws: Option<SocketAddr>,
    pub rpc_index: Option<PathBuf>,
}

impl RawConfig {
    fn parse(self) -> Config {
        Config {
            db_path: self.db_path,
            group_ids: vec![self
                .group_id
                .map(|s| {
                    if s.len() != GROUP_LENGTH * 2 {
                        None
                    } else {
                        let mut value = [0u8; GROUP_LENGTH];
                        let mut is_ok = true;

                        for i in 0..GROUP_LENGTH {
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
                )],
            permission: self.permission.unwrap_or(false),
            only_stable_data: self.only_stable_data.unwrap_or(false),
            p2p_addr: self.p2p_addr.unwrap_or(P2P_ADDR.parse().unwrap()),
            p2p_transport: self
                .p2p_default_transport
                .unwrap_or(P2P_TRANSPORT.to_owned()),
            p2p_allowlist: self.p2p_bootstrap,
            p2p_blocklist: self.p2p_blocklist.unwrap_or(vec![]),
            p2p_allow_peer_list: self
                .p2p_allow_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerAddr::from_hex(s).expect("invalid peer id in p2p allow peer list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            p2p_block_peer_list: self
                .p2p_block_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerAddr::from_hex(s).expect("invalid group id in p2p block peer list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            rpc_addr: self.rpc_addr.unwrap_or(RPC_ADDR.parse().unwrap()),
            rpc_ws: self.rpc_ws,
            rpc_index: self.rpc_index,
        }
    }
}

async fn load_file_string(mut path: PathBuf) -> Result<String> {
    path.push(CONFIG_FILE_NAME);
    debug!("DEBUG-TDN: config file: {:?}", path);
    fs::read_to_string(path).await
}
