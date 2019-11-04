use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use core::primitives::types::RPCParams;
use core::traits::propose::Peer;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Certificate<P: Peer> {
    pub pk: P::PublicKey,
    pub ca: P::PublicKey,
    pub pkc: P::Signature,
}

/// use string to format is better for copy and move
#[derive(Serialize, Deserialize)]
struct CertificateString {
    pk: String,
    ca: String,
    pkc: String,
}

impl<P: Peer> Certificate<P> {
    pub fn new(pk: P::PublicKey, ca: P::PublicKey, pkc: P::Signature) -> Self {
        Self { pk, ca, pkc }
    }

    pub fn certificate(ca_psk: &P::PrivateKey, ca: P::PublicKey, pk: P::PublicKey) -> Self {
        let pkc = P::sign(ca_psk, &(bincode::serialize(&pk).unwrap()));
        Self::new(pk, ca, pkc)
    }

    pub fn certificate_self(psk: &P::PrivateKey, pk: P::PublicKey) -> Self {
        Self::certificate(psk, pk.clone(), pk)
    }

    pub fn verify(ca: &Self) -> bool {
        let pk_vec = {
            let v = bincode::serialize(&ca.pk);
            if v.is_err() {
                return false;
            }
            v.unwrap()
        };
        P::verify(&ca.ca, &pk_vec, &ca.pkc)
    }

    pub fn to_json_string(&self) -> String {
        let ca_string = CertificateString {
            pk: format!("{}", self.pk),
            ca: format!("{}", self.ca),
            pkc: format!("{}", self.pkc),
        };
        serde_json::to_string(&ca_string).unwrap()
    }

    pub fn to_jsonrpc(&self) -> RPCParams {
        json! ({
            "pk": format!("{}", self.pk),
            "ca": format!("{}", self.ca),
            "pkc": format!("{}", self.pkc),
        })
    }

    pub fn from_json_string(s: String) -> Result<Self, ()> {
        let join_type: Result<CertificateString, _> = serde_json::from_str(&s);
        if join_type.is_err() {
            return Err(());
        }
        let ca_str = join_type.unwrap();
        Self::from_string(ca_str.pk, ca_str.ca, ca_str.pkc)
    }

    pub fn from_string(pk: String, ca: String, pkc: String) -> Result<Self, ()> {
        Ok(Self::new(pk.into(), ca.into(), pkc.into()))
    }
}
