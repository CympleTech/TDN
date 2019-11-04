use core::primitives::types::RPCParams;
use core::traits::propose::Peer;

use super::certificate::Certificate;

pub fn add_certificate<P: Peer>(params: RPCParams) -> Result<Certificate<P>, ()> {
    if params.get("pk").is_some() && params.get("ca").is_some() && params.get("pkc").is_some() {
        let pk = params.get("pk").unwrap().as_str().unwrap().to_string();
        let ca = params.get("ca").unwrap().as_str().unwrap().to_string();
        let pkc = params.get("pkc").unwrap().as_str().unwrap().to_string();
        return Certificate::from_string(pk, ca, pkc);
    }
    Err(())
}

pub fn sign_certificate<P: Peer>(
    ca_psk: &P::PrivateKey,
    ca: P::PublicKey,
    params: RPCParams,
) -> Result<Certificate<P>, ()> {
    if params.get("pk").is_some() {
        let str_pk = params.get("pk").unwrap().as_str().unwrap().to_string();
        // 0x... <=> bytes(u8) length
        if str_pk.len() != P::PUBLIC_KEY_LENGTH * 2 + 2 {
            return Err(());
        }
        let pk: P::PublicKey = str_pk.into();
        return Ok(Certificate::certificate(ca_psk, ca, pk));
    }
    Err(())
}
