use bytes::BytesMut;
use serde_json::Value;
use std::path::PathBuf;

use crate::actor::prelude::*;

use super::consts::DEFAULT_STORAGE_DIR_NAME;

pub const DEFAULT_TIMES: u8 = 3;

pub fn try_resend_times<A, M>(addr: Addr<A>, message: M, times: u8) -> Result<(), ()>
where
    A: 'static + Actor + Handler<M>,
    M: 'static + Message + std::marker::Send + Clone,
    <M as Message>::Result: std::marker::Send,
    <A as Actor>::Context: ToEnvelope<A, M>,
{
    if times > 0 {
        match addr.try_send(message.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                try_resend_times(addr, message, times - 1)
            }
        }
    } else {
        Err(())
    }
}

pub fn parse_http_body_json(bytes: &mut BytesMut) -> Result<Value, ()> {
    let mut vec: Vec<u8> = Vec::new();

    for (i, v) in (&bytes).iter().enumerate() {
        if v == &13 || v == &10 {
            vec.push(v.clone())
        } else {
            if vec == [13, 10, 13, 10] {
                return serde_json::from_slice(&bytes.split_off(i)[..]).or(Err(()));
            } else {
                if vec.len() > 0 {
                    vec.clear();
                }
            }
        }
    }

    return Err(());
}

pub fn get_default_storage_path() -> PathBuf {
    #[cfg(feature = "dev")]
    let mut path = PathBuf::from("./");

    #[cfg(not(feature = "dev"))]
    let mut path = if dirs::home_dir().is_some() {
        dirs::home_dir().unwrap()
    } else {
        PathBuf::from("./")
    };

    path.push(DEFAULT_STORAGE_DIR_NAME);
    path
}
