use ed25519_dalek::{Keypair, PublicKey};
use tdn_types::primitives::{PeerKey, Result};

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
pub fn generate_peer(
    language: Language,
    phrase: &str,
    account: u32,
    index: u32,
    passphrase: Option<&str>,
) -> Result<PeerKey> {
    let seed = Mnemonic::from_phrase_in(language, phrase)?.to_seed(passphrase.unwrap_or(""));
    let derive_path = format!("{}/{}'/0/{}", DERIVE_CHAIN, account, index);
    let account = bip32::Ed25519ExtendedPrivKey::derive(&seed, derive_path.as_str())?;
    let sk = account.secret_key;
    let pk: PublicKey = (&sk).into();

    Ok(PeerKey::Ed25519(Keypair {
        public: pk,
        secret: sk,
    }))
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
