use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned as SeDeserializeOwned;
use serde::ser::Serialize as SeSerialize;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::fs;

use chamomile::prelude::Config as P2pConfig;
use tdn_types::{
    group::GroupId,
    primitives::{Peer, PeerId, Result, CONFIG_FILE_NAME, DEFAULT_SECRET, P2P_ADDR, RPC_ADDR},
};

use crate::rpc::RpcConfig;

/// load config from config file.
pub struct Config {
    pub db_path: Option<PathBuf>,
    pub secret: [u8; 32],
    pub group_ids: Vec<GroupId>,
    pub permission: bool,
    pub only_stable_data: bool,

    pub p2p_peer: Peer,
    pub p2p_allowlist: Vec<Peer>,
    pub p2p_blocklist: Vec<IpAddr>,
    pub p2p_allow_peer_list: Vec<PeerId>,
    pub p2p_block_peer_list: Vec<PeerId>,

    pub rpc_addr: SocketAddr,
    pub rpc_ws: Option<SocketAddr>,
    pub rpc_index: Option<PathBuf>,
}

impl Config {
    pub fn split(self) -> ([u8; 32], Vec<GroupId>, P2pConfig, RpcConfig) {
        #[cfg(feature = "single")]
        let delivery_length = 0;
        #[cfg(feature = "multiple")]
        let delivery_length = tdn_types::group::GROUP_BYTES_LENGTH;
        #[cfg(any(feature = "std", feature = "full"))]
        let delivery_length = tdn_types::group::GROUP_BYTES_LENGTH * 2;

        let Config {
            db_path,
            secret,
            group_ids,

            permission,
            only_stable_data,
            p2p_peer,
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
                PathBuf::from("./") // Default is current directory.
            },
            peer: p2p_peer.into(),
            allowlist: p2p_allowlist.iter().map(|p| (p.clone()).into()).collect(),
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

        (secret, group_ids, p2p_config, rpc_config)
    }
}

impl Config {
    pub fn with_addr(p2p_addr: SocketAddr, rpc_addr: SocketAddr) -> Self {
        Config {
            db_path: None,
            secret: DEFAULT_SECRET,
            group_ids: vec![GroupId::default()],
            permission: false,       // default is permissionless
            only_stable_data: false, // default is permissionless
            p2p_peer: Peer::socket(p2p_addr),
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
        let string = load_file_string(PathBuf::from("./")).await;

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

    pub async fn load_save(mut path: PathBuf) -> Self {
        path.push(CONFIG_FILE_NAME);
        if path.exists() {
            if let Ok(string) = fs::read_to_string(path.clone()).await {
                if let Ok(raw_config) = toml::from_str::<RawConfig>(&string) {
                    return raw_config.parse();
                }
            }
        }

        let mut config = Config::default();
        let secret: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20) // 20-length random words.
            .collect();
        config.secret = *blake3::hash(secret.as_bytes()).as_bytes();

        // write to config.toml.
        fs::write(path, generate_config_string(&secret))
            .await
            .unwrap();

        config
    }

    pub async fn load_custom<S: SeSerialize + SeDeserializeOwned>() -> Option<S> {
        let string = load_file_string(PathBuf::from("./")).await;
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
    pub secret: String,
    pub group_id: Option<GroupId>,
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
            secret: *blake3::hash(self.secret.as_bytes()).as_bytes(),
            group_ids: if let Some(g) = self.group_id {
                vec![g]
            } else {
                vec![]
            },
            permission: self.permission.unwrap_or(false),
            only_stable_data: self.only_stable_data.unwrap_or(false),
            p2p_peer: Peer::socket_transport(
                self.p2p_addr.unwrap_or(P2P_ADDR.parse().unwrap()),
                &self.p2p_default_transport.unwrap_or(String::new()),
            ),
            p2p_allowlist: self
                .p2p_bootstrap
                .iter()
                .map(|s| Peer::socket(*s))
                .collect(),
            p2p_blocklist: self.p2p_blocklist.unwrap_or(vec![]),
            p2p_allow_peer_list: self
                .p2p_allow_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerId::from_hex(s).expect("invalid peer id in p2p allow peer list")
                        })
                        .collect()
                })
                .unwrap_or(vec![]),
            p2p_block_peer_list: self
                .p2p_block_peer_list
                .map(|ss| {
                    ss.iter()
                        .map(|s| {
                            PeerId::from_hex(s).expect("invalid group id in p2p block peer list")
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
    Ok(fs::read_to_string(path).await?)
}

fn generate_config_string(secret: &str) -> String {
    format!(
        r#"## TDN Configure.
## Group Now is unique number.
## Example: group_id = 0 # (0 is default group number, as your own dapp.)
group_id = 0

## This will be random string, and you can change.
## if need use secret nonce or seed, it will be useful.
secret = "{}"

## If custom db storage path. uncomment it.
## Default is `$HOME`,  `./` when dev.
## Example: db_path = "./"
#db_path = "../"

## Group Symbol Name.
## If also have group_id and group_symbol, config will use group_id.
## Example: group_symbol = "CypherLink"
group_symbol = "CypherLink"

## App Permission.
## If set true, it is permissioned, and only stable connection;
## if set false, it is permissionless, has stable connection and DHT(p2p) connection.
## Default is false.
## Suggest: if want a permissioned DApp, use `permission = false` and `only_stable_data = true`.
permission = false

## If only receive stable connection's data.
only_stable_data = false

## P2P listen address, default is 0.0.0.0:7364,  uncomment below to change.
p2p_addr = "0.0.0.0:7364"

## P2P transport include: quic, tcp, udt, rtp, default is quic.
p2p_default_transport = "quic"

## P2P bootstrap seed IPs.
## Example: p2p_blocklist = ["1.1.1.1:7364", "192.168.0.1:7364"]
p2p_bootstrap = []

## P2P Blocklist(IP),  uncomment below to change.
## Example: p2p_blocklist = ["1.1.1.1", "192.168.0.1"]
#p2p_blocklist = []

## P2P Allowlist(ID),  uncomment below to change.
## Example: p2p_allow_peer_list = [
##              "55fdd55633c578c7f2fb3e299f3d3bc88f8a9908df448f032c1b5d29db91e8c4",
##              "b9f86efea43016debe9436c2d98fa273789ee81b511bf623e16de0b4c83176a6",
##          ]
#p2p_allow_peer_list = []

## P2P Blocklist (ID),  uncomment below to change.
## Example: p2p_block_peer_list = [
##              "55fdd55633c578c7f2fb3e299f3d3bc88f8a9908df448f032c1b5d29db91e8c4",
##              "b9f86efea43016debe9436c2d98fa273789ee81b511bf623e16de0b4c83176a6",
##          ]
#p2p_block_peer_list = []

## RPC listen address, default is 127.0.0.1:8000, uncomment below to change.
## Example: rpc_addr = "127.0.0.1:8000"
rpc_addr = "127.0.0.1:8000"

## WS listen address, default closed. if need, uncomment below to change.
## Example: rpc_ws = "127.0.0.1:8080"
rpc_ws = "127.0.0.1:8080"

## RPC Service index html body. if has, set path, if not, comment it.
## Example: rpc_index = "/var/www/html/index.html"
"#,
        secret
    )
}
