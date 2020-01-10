use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait Peer {
    type PublicKey: Serialize + DeserializeOwned;
    type SecretKey: Serialize + DeserializeOwned;
    type Signature: Serialize + DeserializeOwned;

    fn sign(
        sk: &Self::SecretKey,
        msg: &Vec<u8>,
    ) -> Result<Self::Signature, Box<dyn std::error::Error>>;

    fn verify(pk: &Self::PublicKey, msg: &Vec<u8>, sign: &Self::Signature) -> bool;
}
