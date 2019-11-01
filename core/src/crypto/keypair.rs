use ed25519_dalek::Keypair;
use ed25519_dalek::PublicKey as EdPublicKey;
use ed25519_dalek::SecretKey as EdPrivateKey;
use ed25519_dalek::Signature as EdSignature;
pub use ed25519_dalek::{
    KEYPAIR_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH as PRIVATE_KEY_LENGTH, SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::{Hash, Hasher};

pub const PRIVATE_HEX_LENGTH: usize = PRIVATE_KEY_LENGTH * 2 + 2; //0x...
pub const PUBLIC_HEX_LENGTH: usize = PUBLIC_KEY_LENGTH * 2 + 2; //0x...
pub const SIGNATURE_HEX_LENGTH: usize = SIGNATURE_LENGTH * 2 + 2; //0x...

#[derive(Deserialize, Serialize)]
pub struct PrivateKey {
    private_key: EdPrivateKey,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PublicKey {
    public_key: EdPublicKey,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Signature {
    signature: EdSignature,
}

impl PrivateKey {
    fn generate_keypair(&self) -> Keypair {
        let public_key_bytes = self.generate_public_key().to_bytes();
        let private_key_bytes = self.to_bytes();
        let mut keypair_bytes: [u8; KEYPAIR_LENGTH] = [0u8; KEYPAIR_LENGTH];
        keypair_bytes[..PRIVATE_KEY_LENGTH].copy_from_slice(&private_key_bytes);
        keypair_bytes[PRIVATE_KEY_LENGTH..].copy_from_slice(&public_key_bytes);

        Keypair::from_bytes(&keypair_bytes).unwrap()
    }

    pub fn generate() -> PrivateKey {
        let mut csprng: OsRng = OsRng::new().unwrap();
        let keypair: Keypair = Keypair::generate(&mut csprng);

        PrivateKey {
            private_key: keypair.secret,
        }
    }

    pub fn generate_public_key(&self) -> PublicKey {
        let public_key = (&self.private_key).into();

        PublicKey { public_key }
    }

    pub fn sign(&self, message: &String) -> Signature {
        let keypair = self.generate_keypair();
        let signature = keypair.sign(message.as_bytes());

        Signature { signature }
    }

    pub fn sign_bytes(&self, bytes: &Vec<u8>) -> Signature {
        let keypair = self.generate_keypair();
        let signature = keypair.sign(bytes);

        Signature { signature }
    }

    pub fn from_bytes(private_key_bytes: &[u8]) -> Option<PrivateKey> {
        if private_key_bytes.len() != PRIVATE_KEY_LENGTH {
            return None;
        }

        let private_key = EdPrivateKey::from_bytes(private_key_bytes);

        if private_key.is_err() {
            return None;
        }

        Some(PrivateKey {
            private_key: private_key.unwrap(),
        })
    }

    pub fn to_bytes(&self) -> [u8; PRIVATE_KEY_LENGTH] {
        self.private_key.to_bytes()
    }

    pub fn as_bytes(&self) -> &[u8; PRIVATE_KEY_LENGTH] {
        self.private_key.as_bytes()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.private_key.as_bytes().to_vec()
    }

    pub fn len(&self) -> usize {
        PRIVATE_KEY_LENGTH
    }
}

impl PublicKey {
    pub fn verify(&self, message: &String, signature: &Signature) -> bool {
        self.public_key
            .verify(message.as_bytes(), &signature.signature)
            .is_ok()
    }

    pub fn verify_bytes(&self, content: &Vec<u8>, signature: &Signature) -> bool {
        self.public_key
            .verify(content, &signature.signature)
            .is_ok()
    }

    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.public_key.to_bytes()
    }

    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        self.public_key.as_bytes()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.public_key.as_bytes().to_vec()
    }

    pub fn from_bytes(public_key_bytes: &[u8]) -> Option<PublicKey> {
        if public_key_bytes.len() != PUBLIC_KEY_LENGTH {
            return None;
        }

        let public_key = EdPublicKey::from_bytes(public_key_bytes);
        if public_key.is_err() {
            return None;
        }

        Some(PublicKey {
            public_key: public_key.unwrap(),
        })
    }

    pub fn len(&self) -> usize {
        PUBLIC_KEY_LENGTH
    }
}

impl Signature {
    pub fn from_bytes(signature_bytes: &[u8]) -> Option<Signature> {
        if signature_bytes.len() != SIGNATURE_LENGTH {
            return None;
        }

        let signature = EdSignature::from_bytes(signature_bytes);
        if signature.is_err() {
            return None;
        }

        Some(Signature {
            signature: signature.unwrap(),
        })
    }

    pub fn len(&self) -> usize {
        SIGNATURE_LENGTH
    }

    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.signature.to_bytes()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.signature.to_bytes().to_vec()
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl From<&String> for PrivateKey {
    fn from(s: &String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; PRIVATE_KEY_LENGTH] = [0; PRIVATE_KEY_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl From<String> for PrivateKey {
    fn from(s: String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; PRIVATE_KEY_LENGTH] = [0; PRIVATE_KEY_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl From<&String> for PublicKey {
    fn from(s: &String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl From<String> for PublicKey {
    fn from(s: String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl From<&String> for Signature {
    fn from(s: &String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl From<String> for Signature {
    fn from(s: String) -> Self {
        let string = &s[2..].to_string();
        let mut bytes: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            bytes[i] = res;
        }
        Self::from_bytes(&bytes).unwrap_or(Default::default())
    }
}

impl Clone for PrivateKey {
    fn clone(&self) -> PrivateKey {
        PrivateKey::from_bytes(&self.to_bytes()).unwrap()
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut hex = String::new();
        hex.extend(self.to_bytes().iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Default for PrivateKey {
    fn default() -> PrivateKey {
        PrivateKey::from_bytes(&[0; PRIVATE_KEY_LENGTH]).unwrap()
    }
}

impl Default for PublicKey {
    fn default() -> PublicKey {
        PublicKey::from_bytes(&[0; PUBLIC_KEY_LENGTH]).unwrap()
    }
}

impl Default for Signature {
    fn default() -> Signature {
        Signature::from_bytes(&[0; SIGNATURE_LENGTH]).unwrap()
    }
}

impl Eq for PublicKey {}

impl Ord for PublicKey {
    fn cmp(&self, other: &PublicKey) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &PublicKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Eq for PrivateKey {}

impl Ord for PrivateKey {
    fn cmp(&self, other: &PrivateKey) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl PartialOrd for PrivateKey {
    fn partial_cmp(&self, other: &PrivateKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &PrivateKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Hash for PrivateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Eq for Signature {}

impl Ord for Signature {
    fn cmp(&self, other: &Signature) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Signature) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}
