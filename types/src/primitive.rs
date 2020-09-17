use lazy_static::lazy_static;
use std::path::PathBuf;

/// P2P default binding addr.
pub const P2P_ADDR: &str = "0.0.0.0:7364";

/// P2P default transport.
pub const P2P_TRANSPORT: &str = "tcp";

/// Layer default binding addr.
pub const LAYER_ADDR: &str = "0.0.0.0:7000";

/// Layer default lower on-off (whether public).
pub const LAYER_PUBLIC_DEFAULT: bool = true;

/// RPC default binding addr.
pub const RPC_ADDR: &str = "127.0.0.1:8000";

/// Configure file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_STORAGE_DIR_NAME: &str = ".tdn";

lazy_static! {
    pub static ref DEFAULT_STORAGE_DIR: PathBuf = {
        #[cfg(feature = "dev")]
        let mut path = PathBuf::from("./");

        #[cfg(not(feature = "dev"))]
        let mut path = if dirs::home_dir().is_some() {
            dirs::home_dir().unwrap()
        } else {
            PathBuf::from("./")
        };

        path.push(DEFAULT_STORAGE_DIR_NAME);
        let _ = std::fs::create_dir_all(&path)
            .expect(&format!("Cannot Build Storage Path: {:?}", path));
        path
    };
}

/// Type: PeerAddr
pub type PeerAddr = chamomile_types::types::PeerId;

/// Type: P2P common Broadcast
pub use chamomile_types::types::Broadcast;

/// Type: P2P stream type.
pub use chamomile_types::message::StreamType;

/// Type: P2P transport stream type.
pub use chamomile_types::types::TransportStream;

pub type Result<T> = std::io::Result<T>;

#[inline]
pub fn new_io_error(info: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, info)
}

#[inline]
pub fn vec_remove_item<T: Eq + PartialEq>(vec: &mut Vec<T>, item: &T) {
    let mut need_remove: Vec<usize> = vec![];
    for (k, i) in vec.iter().enumerate() {
        if i == item {
            need_remove.push(k);
        }
    }

    for i in need_remove.iter().rev() {
        vec.remove(*i);
    }
}

#[inline]
pub fn vec_check_push<T: Eq + PartialEq>(vec: &mut Vec<T>, item: T) {
    for i in vec.iter() {
        if i == &item {
            return;
        }
    }

    vec.push(item);
}

/// Helper: this is the group/layer/rpc handle result in the network.
pub struct HandleResult<'a> {
    /// rpc tasks: [(method, params)].
    pub rpcs: Vec<(&'a str, crate::rpc::RpcParam)>,
    /// group tasks: [GroupSendMessage]
    pub groups: Vec<crate::message::GroupSendMessage>,
    /// layer tasks: [LayerSendMessage]
    pub layers: Vec<crate::message::LayerSendMessage>,
}

impl<'a> HandleResult<'a> {
    pub fn new() -> Self {
        HandleResult {
            rpcs: vec![],
            groups: vec![],
            layers: vec![],
        }
    }
}
