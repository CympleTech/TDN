use byteorder::{BigEndian, ByteOrder};
use serde_derive::{Deserialize, Serialize};

use crate::crypto::keypair::{PublicKey, Signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::primitives::types::GroupID;

use super::content::P2PContent;

pub const HEAD_LENGTH: usize =
    4 + 2 + 32 + PUBLIC_KEY_LENGTH + PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH;
const BEFORE_TO_LENGTH: usize = 4 + 2 + 32 + PUBLIC_KEY_LENGTH;
const BEFORE_SIGN_LENGTH: usize = 4 + 2 + 32 + PUBLIC_KEY_LENGTH + PUBLIC_KEY_LENGTH;

#[derive(Default, Clone, Debug)]
pub struct P2PHead {
    pub len: u32,        //[u8; 4]
    pub ver: u16,        //[u8; 2]
    pub gid: GroupID,    //[u8; 32]
    pub from: PublicKey, //[u8; 32]
    pub to: PublicKey,   //[u8; 32]
    pub sign: Signature, //[u8; 64]
}

impl P2PHead {
    pub fn group(&self) -> &GroupID {
        &self.gid
    }

    pub fn from(&self) -> &PublicKey {
        &self.from
    }

    pub fn to(&self) -> &PublicKey {
        &self.to
    }

    pub fn version(&self) -> u16 {
        self.ver
    }

    pub fn new(ver: u16, gid: GroupID, from: PublicKey, to: PublicKey) -> Self {
        let len = 0;
        let sign = Default::default();

        Self {
            len,
            ver,
            gid,
            from,
            to,
            sign,
        }
    }

    pub fn update_len(&mut self, len: u32) {
        self.len = len
    }

    pub fn update_signature(&mut self, sign: Signature) {
        self.sign = sign;
    }

    pub fn encode(&self) -> [u8; HEAD_LENGTH] {
        let mut bytes = [0u8; HEAD_LENGTH];
        BigEndian::write_u32(&mut bytes, self.len);
        let mut v_bytes = [0u8; 2];
        BigEndian::write_u16(&mut v_bytes, self.ver);
        bytes[4..6].copy_from_slice(&v_bytes);
        bytes[6..38].copy_from_slice(&self.gid.to_bytes());
        bytes[38..BEFORE_TO_LENGTH].copy_from_slice(&self.from.to_bytes());
        bytes[BEFORE_TO_LENGTH..BEFORE_SIGN_LENGTH].copy_from_slice(&self.to.to_bytes());
        bytes[BEFORE_SIGN_LENGTH..HEAD_LENGTH].copy_from_slice(&self.sign.to_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Self {
        let len = BigEndian::read_u32(&bytes[0..4]);
        let ver = BigEndian::read_u16(&bytes[4..6]);
        let gid = {
            let g = GroupID::from_bytes(&bytes[6..38]);
            if g.is_err() {
                Default::default() // TODO Error
            } else {
                g.unwrap()
            }
        };
        let mut from_bytes = [0u8; PUBLIC_KEY_LENGTH];
        from_bytes.copy_from_slice(&bytes[38..BEFORE_TO_LENGTH]);
        let from = PublicKey::from_bytes(&from_bytes).unwrap_or(Default::default());

        let mut to_bytes = [0u8; PUBLIC_KEY_LENGTH];
        to_bytes.copy_from_slice(&bytes[BEFORE_TO_LENGTH..BEFORE_SIGN_LENGTH]);
        let to = PublicKey::from_bytes(&to_bytes).unwrap_or(Default::default());

        let mut sign_bytes = [0u8; SIGNATURE_LENGTH];
        sign_bytes.copy_from_slice(&bytes[BEFORE_SIGN_LENGTH..HEAD_LENGTH]);
        let sign = Signature::from_bytes(&sign_bytes).unwrap_or(Default::default());

        Self {
            len,
            ver,
            gid,
            from,
            to,
            sign,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct P2PBody(pub P2PContent);
