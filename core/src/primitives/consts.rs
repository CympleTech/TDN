pub const K_BUCKET: usize = 8;
pub const LOW_WATERMARK: usize = 200;
pub const HIGH_WATERMARK: usize = 1 * 1024 * 1024 + 200; // 1MB + HEADER
pub const DEFAULT_STORAGE_DIR_NAME: &'static str = ".tea";
pub const P2P_CACHE_DIR_NAME: &'static str = "p2p_cache";
pub const P2P_DEFAULT_SOCKET: &'static str = "0.0.0.0:7364";
pub const RPC_DEFAULT_SOCKET: &'static str = "0.0.0.0:3030";
