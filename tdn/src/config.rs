use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use serde::de::DeserializeOwned as SeDeserializeOwned;
use serde::ser::Serialize as SeSerialize;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use chamomile::prelude::Config as P2pConfig;
use tdn_types::{
    group::GroupId,
    primitives::{Peer, PeerId, Result, CONFIG_FILE_NAME, DEFAULT_SECRET, P2P_ADDR, RPC_HTTP},
};

use crate::rpc::{ChannelAddr, RpcConfig};

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

    pub rpc_http: Option<SocketAddr>,
    pub rpc_ws: Option<SocketAddr>,
    pub rpc_channel: Option<ChannelAddr>,
    pub rpc_index: Option<PathBuf>,
}

impl Config {
    pub fn split(self) -> ([u8; 32], Vec<GroupId>, P2pConfig, RpcConfig) {
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

            rpc_http,
            rpc_ws,
            rpc_channel,
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
            permission,
            only_stable_data,
            delivery_length,
        };

        let rpc_config = RpcConfig {
            http: rpc_http,
            ws: rpc_ws,
            channel: rpc_channel,
            index: rpc_index,
        };

        (secret, group_ids, p2p_config, rpc_config)
    }
}

impl Config {
    pub fn with_addr(p2p_addr: SocketAddr, rpc_http: SocketAddr) -> Self {
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

            rpc_http: Some(rpc_http),
            rpc_ws: None,
            rpc_channel: None,
            rpc_index: None,
        }
    }

    pub fn default() -> Self {
        Config::with_addr(P2P_ADDR.parse().unwrap(), RPC_HTTP.parse().unwrap())
    }

    pub async fn load(path: PathBuf) -> Self {
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

    pub async fn load_save(mut path: PathBuf, mut config: Config) -> Result<Self> {
        path.push(CONFIG_FILE_NAME);
        if path.exists() {
            if let Ok(string) = fs::read_to_string(path.clone()).await {
                let raw_config = toml::from_str::<RawConfig>(&string)?;
                return Ok(raw_config.parse());
            }
        }

        let mut rng = ChaChaRng::from_entropy();
        let mut key = [0u8; 20];
        rng.fill_bytes(&mut key);
        let secret = String::from_utf8(key.to_vec()).unwrap_or("".to_owned());
        config.secret = *blake3::hash(secret.as_bytes()).as_bytes();

        // write to config.toml.
        fs::write(path, generate_config_string(&config, &secret)).await?;

        Ok(config)
    }

    pub async fn load_custom<S: SeSerialize + SeDeserializeOwned>(path: PathBuf) -> Option<S> {
        let string = load_file_string(path).await;
        match string {
            Ok(string) => toml::from_str::<S>(&string).ok(),
            Err(_) => None,
        }
    }

    pub async fn append_custom(mut path: PathBuf, s: &str) -> Result<()> {
        path.push(CONFIG_FILE_NAME);
        let mut file = OpenOptions::new().append(true).open(path).await?;
        file.write(s.as_bytes()).await?;
        Ok(file.flush().await?)
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
    pub permission: Option<bool>,
    pub only_stable_data: Option<bool>,

    pub p2p_addr: Option<SocketAddr>,
    pub p2p_default_transport: Option<String>,
    pub p2p_bootstrap: Vec<SocketAddr>,
    pub p2p_blocklist: Option<Vec<IpAddr>>,
    pub p2p_allow_peer_list: Option<Vec<String>>,
    pub p2p_block_peer_list: Option<Vec<String>>,

    pub rpc_http: Option<SocketAddr>,
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
            rpc_http: Some(self.rpc_http.unwrap_or(RPC_HTTP.parse().unwrap())),
            rpc_ws: self.rpc_ws,
            rpc_channel: None,
            rpc_index: self.rpc_index,
        }
    }
}

async fn load_file_string(mut path: PathBuf) -> Result<String> {
    path.push(CONFIG_FILE_NAME);
    Ok(fs::read_to_string(path).await?)
}

fn generate_config_string(config: &Config, secret: &str) -> String {
    let group_id_str = format!(
        r#"## Application unique GroupId number.
## Example: group_id = 0 # (0 is default group number, as your own dapp.)
group_id = {}
"#,
        if config.group_ids.len() > 0 {
            config.group_ids[0]
        } else {
            0
        }
    );

    let secret_str = format!(
        r#"## This will be random string, and you can change.
## if need use secret nonce or seed, it will be useful.
secret = "{}"
"#,
        secret
    );

    let db_path_str = match &config.db_path {
        Some(path) => format!(
            r#"## If custom db storage path. uncomment it.
## Default is `$HOME`,  `./` when dev.
## Example: db_path = "./"
db_path = {:?}
"#,
            path
        ),
        None => format!(
            r#"## If custom db storage path. uncomment it.
## Default is `$HOME`,  `./` when dev.
## Example: db_path = "./"
#db_path = "./"
"#
        ),
    };

    let permission_str = format!(
        r#"## App Permission.
## If set true, it is permissioned, and only stable connection;
## if set false, it is permissionless, has stable connection and DHT(p2p) connection.
## Default is false.
## Suggest: if want a permissioned DApp, use `permission = false` and `only_stable_data = true`.
permission = {}
"#,
        config.permission
    );

    let only_stable_data_str = format!(
        r#"## If only receive stable connection's data.
only_stable_data = {}
"#,
        config.only_stable_data
    );

    let peer_addr_str = format!(
        r#"## P2P listen address, default is 0.0.0.0:7364,  uncomment below to change.
p2p_addr = "{}"
"#,
        config.p2p_peer.socket
    );

    let p2p_default_transport_str = format!(
        r#"## P2P transport include: quic, tcp, udt, rtp, default is quic.
p2p_default_transport = "{}"
"#,
        config.p2p_peer.transport.to_str()
    );

    let p2p_bootstrap_str = format!(
        r#"## P2P bootstrap seed IPs.
## Example: p2p_blocklist = ["1.1.1.1:7364", "192.168.0.1:7364"]
p2p_bootstrap = [{}]
"#,
        config
            .p2p_allowlist
            .iter()
            .map(|p| format!("\"{}\"", p.socket.to_string()))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let p2p_block_str = format!(
        r#"## P2P Blocklist(IP),  uncomment below to change.
## Example: p2p_blocklist = ["1.1.1.1", "192.168.0.1"]
p2p_blocklist = [{}]
"#,
        config
            .p2p_blocklist
            .iter()
            .map(|p| format!("\"{}\"", p.to_string()))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let p2p_allow_peer_str = format!(
        r#"## P2P Allowlist(ID),  uncomment below to change.
## Example: p2p_allow_peer_list = [
##              "55fdd55633c578c7f2fb3e299f3d3bc88f8a9908",
##              "b9f86efea43016debe9436c2d98fa273789ee81b",
##          ]
p2p_allow_peer_list = [{}]
"#,
        config
            .p2p_allow_peer_list
            .iter()
            .map(|p| format!("\"{}\"", p.to_hex()))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let p2p_block_peer_str = format!(
        r#"## P2P Blocklist (ID),  uncomment below to change.
## Example: p2p_block_peer_list = [
##              "55fdd55633c578c7f2fb3e299f3d3bc88f8a9908",
##              "b9f86efea43016debe9436c2d98fa273789ee81b",
##          ]
p2p_block_peer_list = [{}]
"#,
        config
            .p2p_block_peer_list
            .iter()
            .map(|p| format!("\"{}\"", p.to_hex()))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let rpc_http_str = match &config.rpc_http {
        Some(addr) => format!(
            r#"## RPC HTTP listen address, default is 127.0.0.1:7365, uncomment below to change.
## Example: rpc_http = "127.0.0.1:7365"
rpc_http = "{}"
"#,
            addr
        ),
        None => format!(
            r#"## RPC WS listen address, default closed. if need, uncomment below to change.
## Example: rpc_http = "127.0.0.1:7365"
#rpc_http = "{}"
"#,
            RPC_HTTP
        ),
    };

    let rpc_ws_str = match &config.rpc_ws {
        Some(addr) => format!(
            r#"## WS listen address, default closed. if need, uncomment below to change.
## Example: rpc_ws = "127.0.0.1:7366"
rpc_ws = "{}"
"#,
            addr
        ),
        None => format!(
            r#"## WS listen address, default closed. if need, uncomment below to change.
## Example: rpc_ws = "127.0.0.1:7366"
#rpc_ws = ""
"#
        ),
    };

    let rpc_index_str = match &config.rpc_index {
        Some(path) => format!(
            r#"## RPC Service index html body. if has, set path, if not, comment it.
## Example: rpc_index = "/var/www/html/index.html"
rpc_index = {:?}
"#,
            path
        ),
        None => format!(
            r#"## RPC Service index html body. if has, set path, if not, comment it.
## Example: rpc_index = "/var/www/html/index.html"
#rpc_index = ""
"#
        ),
    };

    format!(
        r#"## TDN Configure.
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
"#,
        group_id_str,
        secret_str,
        db_path_str,
        permission_str,
        only_stable_data_str,
        peer_addr_str,
        p2p_default_transport_str,
        p2p_bootstrap_str,
        p2p_block_str,
        p2p_allow_peer_str,
        p2p_block_peer_str,
        rpc_http_str,
        rpc_ws_str,
        rpc_index_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CustomConfig {
        pub name: String,
        pub info: Option<String>,
    }

    #[test]
    fn test_config() {
        let path = PathBuf::from("./.test_config");
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async {
            let mut config = Config::default();
            config.db_path = Some(path.clone());
            config.rpc_http = Some("127.0.0.1:8000".parse().unwrap());
            config.p2p_allowlist = vec![
                Peer::socket("1.1.1.1:7364".parse().unwrap()),
                Peer::socket("2.2.2.2:7364".parse().unwrap()),
            ];
            config.p2p_blocklist = vec!["3.3.3.3".parse().unwrap()];
            config.p2p_allow_peer_list = vec![PeerId::default()];
            config.p2p_block_peer_list = vec![
                PeerId::from_hex("55fdd55633c578c7f2fb3e299f3d3bc88f8a9908").unwrap(),
                PeerId::from_hex("b9f86efea43016debe9436c2d98fa273789ee81b").unwrap(),
            ];
            config.rpc_ws = Some("127.0.0.1:8001".parse().unwrap());
            config.rpc_index = Some(PathBuf::from("/var/www/html/index.html"));

            let config = Config::load_save(path.clone(), config).await.unwrap();
            let new_config = Config::load_save(path.clone(), config).await.unwrap();
            assert_eq!(new_config.db_path, Some(path.clone()));
            assert_eq!(new_config.rpc_http, Some("127.0.0.1:8000".parse().unwrap()));
            assert_eq!(
                new_config.p2p_allowlist[0],
                Peer::socket("1.1.1.1:7364".parse().unwrap())
            );
            assert_eq!(
                new_config.p2p_block_peer_list[0],
                PeerId::from_hex("55fdd55633c578c7f2fb3e299f3d3bc88f8a9908").unwrap(),
            );
            assert!(new_config.rpc_ws.is_some());
            assert!(new_config.rpc_index.is_some());
            std::fs::remove_dir_all(path).unwrap();
        });
    }

    #[test]
    fn test_custom_config() {
        let path = PathBuf::from("./.test_custom_config");
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async {
            let _config = Config::load_save(path.clone(), Config::default()).await;

            let raw = format!(
                r#"## Custom Config
## Test custom name
name = "{}"

## Test custom info
info = "{}"
"#,
                "cympletech", "custom_info"
            );

            Config::append_custom(path.clone(), &raw).await.unwrap();
            let custom_config: CustomConfig = Config::load_custom(path.clone()).await.unwrap();
            assert_eq!(custom_config.name, "cympletech".to_owned());
            assert_eq!(custom_config.info, Some("custom_info".to_owned()));
            std::fs::remove_dir_all(path).unwrap();
        });
    }
}
