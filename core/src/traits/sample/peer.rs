use serde_derive::{Deserialize, Serialize};

use crate::crypto::keypair::{
    PrivateKey, PublicKey, Signature, PRIVATE_KEY_LENGTH, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
use crate::traits::propose::Peer;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct NetworkPeer {
    pk: PublicKey,
    psk: PrivateKey,
}

impl NetworkPeer {
    pub fn new(pk: PublicKey, psk: PrivateKey) -> Self {
        NetworkPeer { pk, psk }
    }
}

impl Peer for NetworkPeer {
    type PrivateKey = PrivateKey;
    type PublicKey = PublicKey;
    type Signature = Signature;
    const PRIVATE_KEY_LENGTH: usize = PRIVATE_KEY_LENGTH;
    const PUBLIC_KEY_LENGTH: usize = PUBLIC_KEY_LENGTH;
    const SIGNATURE_KEY_LENGTH: usize = SIGNATURE_LENGTH;

    fn pk(&self) -> &Self::PublicKey {
        &self.pk
    }

    fn generate() -> (PublicKey, PrivateKey) {
        let private_key = PrivateKey::generate();
        let public_key = private_key.generate_public_key();

        (public_key, private_key)
    }

    fn sign(psk: &Self::PrivateKey, data: &Vec<u8>) -> Self::Signature {
        psk.sign_bytes(data)
    }

    fn verify(pk: &Self::PublicKey, data: &Vec<u8>, signature: &Self::Signature) -> bool {
        pk.verify_bytes(data, signature)
    }

    fn public_key_from_bytes(bytes: &[u8]) -> Option<Self::PublicKey> {
        PublicKey::from_bytes(bytes)
    }

    fn public_key_to_bytes(pk: &Self::PublicKey) -> Vec<u8> {
        PublicKey::to_bytes(pk).to_vec()
    }

    fn private_key_from_bytes(bytes: &[u8]) -> Option<Self::PrivateKey> {
        PrivateKey::from_bytes(bytes)
    }

    fn private_key_to_bytes(psk: &Self::PrivateKey) -> Vec<u8> {
        PrivateKey::to_bytes(psk).to_vec()
    }
}
