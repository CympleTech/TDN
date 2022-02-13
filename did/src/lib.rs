use serde::{Deserialize, Serialize};

pub use ed25519_dalek::Keypair;

use ed25519_dalek::{PublicKey, Signature, Signer, Verifier};
use signature::Signature as _;
use tdn_types::primitives::{PeerId, Result};

mod bip32;
mod bip39;
mod bip44;
mod error;
mod language;

pub use bip32::Ed25519ExtendedPrivKey;
#[cfg(feature = "secp256k1")]
pub use bip32::Secp256k1ExtendedPrivKey;

#[cfg(feature = "secp256k1")]
pub use secp256k1;

pub use bip39::{Count, Mnemonic};
pub use error::Error;
pub use language::Language;

pub const PROOF_LENGTH: usize = 64; // use ed25519 signaure length.
const DERIVE_CHAIN: &'static str = "m/44'/7364'";
#[cfg(feature = "secp256k1")]
const ETH_CHAIN: &'static str = "m/44'/60'";
#[cfg(feature = "secp256k1")]
const BTC_CHAIN: &'static str = "m/44'/0'";

/// generate mnemonic codes by language & words number.
#[cfg(feature = "rand")]
pub fn generate_mnemonic(language: Language, count: Count) -> String {
    Mnemonic::generate_in(language, count).phrase().to_string()
}

/// generate tdn id (ed25519) by mnemonic codes, account, index.
pub fn generate_id(
    language: Language,
    phrase: &str,
    account: u32,
    index: u32,
    passphrase: Option<&str>,
) -> Result<Keypair> {
    let seed = Mnemonic::from_phrase_in(language, phrase)?.to_seed(passphrase.unwrap_or(""));
    let derive_path = format!("{}/{}'/0/{}", DERIVE_CHAIN, account, index);
    let account = bip32::Ed25519ExtendedPrivKey::derive(&seed, derive_path.as_str())?;
    let sk = account.secret_key;
    let pk: PublicKey = (&sk).into();

    Ok(Keypair {
        public: pk,
        secret: sk,
    })
}

/// generate ETH secret_key by mnemonic codes, account, index.
#[cfg(feature = "secp256k1")]
pub fn generate_eth_account(
    language: Language,
    phrase: &str,
    account: u32,
    index: u32,
    passphrase: Option<&str>,
) -> Result<secp256k1::SecretKey> {
    let seed = Mnemonic::from_phrase_in(language, phrase)?.to_seed(passphrase.unwrap_or(""));
    let derive_path = format!("{}/{}'/0/{}", ETH_CHAIN, account, index);
    let account = Secp256k1ExtendedPrivKey::derive(&seed, derive_path.as_ref())?;
    Ok(account.secret_key)
}

/// generate ETH secret_key by mnemonic codes, account, index.
#[cfg(feature = "secp256k1")]
pub fn generate_btc_account(
    language: Language,
    phrase: &str,
    account: u32,
    index: u32,
    passphrase: Option<&str>,
) -> Result<secp256k1::SecretKey> {
    let seed = Mnemonic::from_phrase_in(language, phrase)?.to_seed(passphrase.unwrap_or(""));
    let derive_path = format!("{}/{}'/0/{}", BTC_CHAIN, account, index);
    let account = Secp256k1ExtendedPrivKey::derive(&seed, derive_path.as_ref())?;
    Ok(account.secret_key)
}

#[derive(Default, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
pub struct Proof(pub Vec<u8>);

impl Proof {
    pub fn prove(kp: &Keypair, maddr: &PeerId, raddr: &PeerId) -> Proof {
        let mut bytes = vec![];
        bytes.extend(&maddr.0);
        bytes.extend(&raddr.0);
        Proof(kp.sign(&bytes).as_bytes().to_vec())
    }

    pub fn verify(&self, pk: &PublicKey, maddr: &PeerId, raddr: &PeerId) -> Result<()> {
        if self.0.len() != PROOF_LENGTH {
            return Err(anyhow::anyhow!("proof length failure!"));
        }
        let sign = Signature::from_bytes(&self.0)?;

        let mut bytes = vec![];
        bytes.extend(&maddr.0);
        bytes.extend(&raddr.0);

        Ok(pk.verify(&bytes, &sign)?)
    }

    pub fn prove_bytes(kp: &Keypair, bytes: &[u8]) -> Proof {
        Proof(kp.sign(bytes).as_bytes().to_vec())
    }

    pub fn verify_bytes(&self, pk: &PublicKey, bytes: &[u8]) -> Result<()> {
        if self.0.len() != PROOF_LENGTH {
            return Err(anyhow::anyhow!("proof length failure!"));
        }
        let sign = Signature::from_bytes(&self.0)?;
        Ok(pk.verify(bytes, &sign)?)
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }

    pub fn from_hex(s: &str) -> Result<Proof> {
        let s = s.to_string();
        if s.len() % 2 == 1 {
            return Err(anyhow::anyhow!("Hex is invalid!"));
        }

        let mut bytes = vec![];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)?;
            bytes.push(res);
        }

        Ok(Proof(bytes))
    }
}
