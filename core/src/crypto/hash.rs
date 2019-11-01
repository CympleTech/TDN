use serde_derive::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Sha3_512};
use std::cmp::Ordering;
use std::convert::AsRef;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};

type Sha256 = Sha3_256;
type Sha512 = Sha3_512;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct H256 {
    value: [u8; 32],
}

#[derive(Clone)]
pub struct H512 {
    value: [u8; 64],
}

impl H256 {
    pub fn new(content: &[u8]) -> H256 {
        let mut h: H256 = Default::default();
        let mut hasher = Sha256::default();
        hasher.input(content);
        h.value.copy_from_slice(&hasher.result()[..]);
        h
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.value.to_vec()
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, ()> {
        if data.len() != 32 {
            return Err(());
        }

        let mut value: [u8; 32] = [0; 32];
        value.copy_from_slice(&data[..]);

        Ok(Self { value })
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn from_string(s: &String) -> Result<Self, ()> {
        let string = s[2..].to_string();

        if string.len() != 64 {
            return Err(());
        }

        let mut value: [u8; 32] = [0; 32];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            value[i] = res;
        }
        Ok(Self { value })
    }

    pub fn from_str(s: &str) -> Result<Self, ()> {
        let string = &s[2..].to_string();

        if string.len() != 64 {
            return Err(());
        }

        let mut value: [u8; 32] = [0; 32];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            value[i] = res;
        }
        Ok(Self { value })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() != 32 {
            return Err(());
        }

        let mut value: [u8; 32] = Default::default();
        value.copy_from_slice(bytes);
        Ok(Self { value })
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.value.clone()
    }
}

impl H512 {
    pub fn new(content: &[u8]) -> H512 {
        let mut h: H512 = Default::default();
        let mut hasher = Sha512::default();
        hasher.input(content);
        h.value.copy_from_slice(&hasher.result()[..]);
        h
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.value.to_vec()
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, ()> {
        if data.len() != 64 {
            return Err(());
        }

        let mut value = [0u8; 64];
        value.copy_from_slice(&data[..]);
        Ok(Self { value })
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn from_string(s: &String) -> Result<Self, ()> {
        let string = s[2..].to_string();

        if string.len() != 128 {
            return Err(());
        }

        let mut value = [0u8; 64];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            value[i] = res;
        }
        Ok(Self { value })
    }

    pub fn from_str(s: &str) -> Result<Self, ()> {
        let string = &s[2..].to_string();
        if string.len() != 128 {
            return Err(());
        }

        let mut value = [0u8; 64];

        for i in 0..(string.len() / 2) {
            let res = u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap();
            value[i] = res;
        }
        Ok(Self { value })
    }
}

impl Display for H256 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.value.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Debug for H256 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.value.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Display for H512 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.value.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

impl Debug for H512 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.value.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

// derive Default
impl Default for H512 {
    fn default() -> H512 {
        let value = [0u8; 64];
        H512 { value }
    }
}

// derive Debug

// derive Serialize

impl Eq for H256 {}

impl Ord for H256 {
    fn cmp(&self, other: &H256) -> Ordering {
        self.to_vec().cmp(&other.to_vec())
    }
}

impl PartialOrd for H256 {
    fn partial_cmp(&self, other: &H256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for H256 {
    fn eq(&self, other: &H256) -> bool {
        self.to_vec() == other.to_vec()
    }
}

impl Eq for H512 {}

impl Ord for H512 {
    fn cmp(&self, other: &H512) -> Ordering {
        self.to_vec().cmp(&other.to_vec())
    }
}

impl PartialOrd for H512 {
    fn partial_cmp(&self, other: &H512) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for H512 {
    fn eq(&self, other: &H512) -> bool {
        self.to_vec() == other.to_vec()
    }
}

impl Hash for H256 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

impl AsRef<[u8]> for H512 {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}
