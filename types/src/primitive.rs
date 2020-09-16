use lazy_static::lazy_static;
use serde_json::Value;
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

// Type: RPC Param
pub type RpcParam = Value;
pub use serde_json::json;

/// Type: PeerAddr
pub type PeerAddr = chamomile_types::types::PeerId;

/// Type: P2P common Broadcast
pub use chamomile_types::types::Broadcast;

/// Type: P2P stream type.
pub use chamomile_types::message::StreamType;

/// Type: P2P transport stream type.
pub use chamomile_types::types::TransportStream;
