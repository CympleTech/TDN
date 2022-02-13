use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::fmt;
use std::ops::Deref;
use zeroize::Zeroize;

#[cfg(feature = "secp256k1")]
use secp256k1::{PublicKey as Secp256k1PublicKey, SecretKey as Secp256k1SecretKey};

use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::{PublicKey as Ed25519PublicKey, SecretKey as Ed25519SecretKey};

use crate::bip44::{ChildNumber, IntoDerivationPath};
use crate::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Protected([u8; 32]);

impl Zeroize for Protected {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<Data: AsRef<[u8]>> From<Data> for Protected {
    fn from(data: Data) -> Protected {
        let mut buf = [0u8; 32];

        buf.copy_from_slice(data.as_ref());

        Protected(buf)
    }
}

impl Deref for Protected {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for Protected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Protected")
    }
}

#[cfg(feature = "secp256k1")]
pub struct Secp256k1ExtendedPrivKey {
    pub secret_key: Secp256k1SecretKey,
    chain_code: Protected,
}

#[cfg(feature = "secp256k1")]
impl Secp256k1ExtendedPrivKey {
    /// Attempts to derive an extended private key from a path.
    pub fn derive<Path>(seed: &[u8], path: Path) -> Result<Secp256k1ExtendedPrivKey, Error>
    where
        Path: IntoDerivationPath,
    {
        let mut hmac: Hmac<Sha512> =
            Hmac::new_from_slice(b"Bitcoin seed").expect("seed is always correct.");
        hmac.update(seed);

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.split_at(32);

        let mut sk = Secp256k1ExtendedPrivKey {
            secret_key: Secp256k1SecretKey::from_slice(secret_key).map_err(Error::Secp256k1)?,
            chain_code: Protected::from(chain_code),
        };

        for child in path.into()?.as_ref() {
            sk = sk.child(*child)?;
        }

        Ok(sk)
    }

    pub fn child(&self, child: ChildNumber) -> Result<Secp256k1ExtendedPrivKey, Error> {
        let mut hmac: Hmac<Sha512> =
            Hmac::new_from_slice(&self.chain_code).map_err(|_| Error::InvalidChildNumber)?;
        let secp = secp256k1::Secp256k1::new();

        if child.is_normal() {
            hmac.update(
                &Secp256k1PublicKey::from_secret_key(&secp, &self.secret_key).serialize()[..],
            );
        } else {
            hmac.update(&[0]);
            hmac.update(self.secret_key.as_ref());
        }

        hmac.update(&child.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.split_at(32);

        let mut secret_key =
            Secp256k1SecretKey::from_slice(&secret_key).map_err(Error::Secp256k1)?;
        secret_key
            .add_assign(self.secret_key.as_ref())
            .map_err(Error::Secp256k1)?;

        Ok(Secp256k1ExtendedPrivKey {
            secret_key,
            chain_code: Protected::from(&chain_code),
        })
    }
}

#[cfg(all(test, feature = "secp256k1"))]
impl std::str::FromStr for Secp256k1ExtendedPrivKey {
    type Err = Error;

    fn from_str(xprv: &str) -> Result<Secp256k1ExtendedPrivKey, Error> {
        let data = bs58::decode(xprv)
            .into_vec()
            .map_err(|_| Error::InvalidExtendedPrivKey)?;

        if data.len() != 82 {
            return Err(Error::InvalidExtendedPrivKey);
        }

        Ok(Secp256k1ExtendedPrivKey {
            chain_code: Protected::from(&data[13..45]),
            secret_key: Secp256k1SecretKey::from_slice(&data[46..78])
                .map_err(|e| Error::Secp256k1(e))?,
        })
    }
}

pub struct Ed25519ExtendedPrivKey {
    pub secret_key: Ed25519SecretKey,
    chain_code: Protected,
}

impl Ed25519ExtendedPrivKey {
    /// Attempts to derive an extended private key from a path.
    pub fn derive<Path>(seed: &[u8], path: Path) -> Result<Ed25519ExtendedPrivKey, Error>
    where
        Path: IntoDerivationPath,
    {
        let mut hmac: Hmac<Sha512> =
            Hmac::new_from_slice(b"ed25519 seed").expect("seed is always correct.");
        hmac.update(seed);

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.split_at(32);

        let mut sk = Ed25519ExtendedPrivKey {
            secret_key: Ed25519SecretKey::from_bytes(secret_key)
                .map_err(|e| Error::Ed25519(e.to_string()))?,
            chain_code: Protected::from(chain_code),
        };

        for child in path.into()?.as_ref() {
            sk = sk.child(*child)?;
        }

        Ok(sk)
    }

    pub fn secret(&self) -> [u8; 32] {
        self.secret_key.to_bytes()
    }

    pub fn child(&self, child: ChildNumber) -> Result<Ed25519ExtendedPrivKey, Error> {
        let mut hmac: Hmac<Sha512> =
            Hmac::new_from_slice(&self.chain_code).map_err(|_| Error::InvalidChildNumber)?;

        if child.is_normal() {
            hmac.update(Ed25519PublicKey::from(&self.secret_key).as_bytes());
        } else {
            hmac.update(&[0]);
            hmac.update(self.secret_key.as_bytes());
        }

        hmac.update(&child.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.split_at(32);

        let secret_key_t =
            Ed25519SecretKey::from_bytes(secret_key).map_err(|e| Error::Ed25519(e.to_string()))?;
        let l = Scalar::from_bits(secret_key_t.to_bytes());
        let r = Scalar::from_bits(self.secret_key.to_bytes());
        let v = l + r;
        if v == Scalar::zero() {
            return Err(Error::InvalidExtendedPrivKey);
        }
        let secret_key = Ed25519SecretKey::from_bytes(v.as_bytes())
            .map_err(|e| Error::Ed25519(e.to_string()))?;

        Ok(Ed25519ExtendedPrivKey {
            secret_key,
            chain_code: Protected::from(&chain_code),
        })
    }
}

#[cfg(all(test, feature = "secp256k1"))]
mod tests_secp256k1 {
    use super::*;
    use crate::{Language, Mnemonic};
    use ethsign::SecretKey;
    use std::str::FromStr;

    #[test]
    fn bip39_to_address() {
        let phrase = "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside";

        let expected_secret_key = b"\xff\x1e\x68\xeb\x7b\xf2\xf4\x86\x51\xc4\x7e\xf0\x17\x7e\xb8\x15\x85\x73\x22\x25\x7c\x58\x94\xbb\x4c\xfd\x11\x76\xc9\x98\x93\x14";
        let expected_address: &[u8] =
            b"\x63\xF9\xA9\x2D\x8D\x61\xb4\x8a\x9f\xFF\x8d\x58\x08\x04\x25\xA3\x01\x2d\x05\xC8";

        let mnemonic = Mnemonic::from_phrase_in(Language::English, phrase).unwrap();
        let seed = mnemonic.to_seed("");

        let account = Secp256k1ExtendedPrivKey::derive(&seed, "m/44'/60'/0'/0/0").unwrap();

        assert_eq!(
            expected_secret_key,
            account.secret_key.as_ref(),
            "Secret key is invalid"
        );

        let secret_key = SecretKey::from_raw(account.secret_key.as_ref()).unwrap();
        let public_key = secret_key.public();

        assert_eq!(expected_address, public_key.address(), "Address is invalid");

        // Test child method
        let account = Secp256k1ExtendedPrivKey::derive(&seed, "m/44'/60'/0'/0")
            .unwrap()
            .child(ChildNumber::from_str("0").unwrap())
            .unwrap();

        assert_eq!(
            expected_secret_key,
            account.secret_key.as_ref(),
            "Secret key is invalid"
        );

        let secret_key = SecretKey::from_raw(account.secret_key.as_ref()).unwrap();
        let public_key = secret_key.public();

        assert_eq!(expected_address, public_key.address(), "Address is invalid");

        let account = Secp256k1ExtendedPrivKey::derive(&seed, "m/44'/60'/0'/0")
            .unwrap()
            .child(ChildNumber::from_str("1").unwrap())
            .unwrap();

        let secret_key = SecretKey::from_raw(account.secret_key.as_ref()).unwrap();
        let public_key = secret_key.public();

        let expected_address: &[u8] =
            b"\x43\xBa\x63\x12\x81\x66\xa0\x6B\x1C\xe0\xe4\x3B\xaf\xFE\x99\xf3\xdE\x27\x98\x68";
        assert_eq!(expected_address, public_key.address(), "Address is invalid");

        let account = Secp256k1ExtendedPrivKey::derive(&seed, "m/44'/60'/0'/0")
            .unwrap()
            .child(ChildNumber::from_str("2").unwrap())
            .unwrap();

        let secret_key = SecretKey::from_raw(account.secret_key.as_ref()).unwrap();
        let public_key = secret_key.public();
        let expected_address: &[u8] =
            b"\x0a\x71\x5f\xfE\x21\x56\x79\xae\xF9\x40\xbF\x12\x9D\xf2\x9A\xCA\x07\x0A\xBF\xF8";
        assert_eq!(expected_address, public_key.address(), "Address is invalid");
    }
}

#[cfg(test)]
mod tests_ed25519 {
    use super::*;
    use crate::{Language, Mnemonic};
    use ed25519_dalek::PublicKey;

    #[test]
    fn bip39_to_address() {
        let phrase =
            "voice become refuse remove ordinary recall humble purity shock fetch open scale knee above axis blossom differ bamboo ski drip forest fade ill door";

        let mnemonic = Mnemonic::from_phrase_in(Language::English, phrase).unwrap();
        let seed = mnemonic.to_seed("");

        let account = Ed25519ExtendedPrivKey::derive(&seed, "m/44'/354'/0'/0/0").unwrap();
        let public_key: PublicKey = (&account.secret_key).into();

        println!("SecretKey 1: {:?}", hex::encode(&account.secret()));
        println!("PublicKey 1: {:?}", hex::encode(&public_key));

        // Test child method
        let account = Ed25519ExtendedPrivKey::derive(&seed, "m/44'/354'/0'/0/1").unwrap();

        let public_key: PublicKey = (&account.secret_key).into();

        println!("SecretKey 2: {:?}", hex::encode(&account.secret()));
        println!("PublicKey 2: {:?}", hex::encode(&public_key));

        // Test child method
        let account = Ed25519ExtendedPrivKey::derive(&seed, "m/44'/7364'/0'/0/0").unwrap();

        let public_key: PublicKey = (&account.secret_key).into();

        println!("SecretKey 3: {:?}", hex::encode(&account.secret()));
        println!("PublicKey 3: {:?}", hex::encode(&public_key));

        // Test child method
        let account = Ed25519ExtendedPrivKey::derive(&seed, "m/44'/7364'/0'/0/1").unwrap();

        let public_key: PublicKey = (&account.secret_key).into();

        println!("SecretKey 4: {:?}", hex::encode(&account.secret()));
        println!("PublicKey 4: {:?}", hex::encode(&public_key));
    }
}
