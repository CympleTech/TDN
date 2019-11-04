use core::actor::prelude::{Message, Recipient};
use core::crypto::hash::H256;

pub type ID = H256;

/// read data message
#[derive(Clone)]
pub struct Read(pub Recipient<ReadResult>, pub usize, pub ID);

impl Message for Read {
    type Result = ();
}

#[derive(Clone)]
pub struct ReadResult(pub usize, pub Option<Vec<u8>>);

impl Message for ReadResult {
    type Result = ();
}

/// write data message
#[derive(Clone)]
pub struct Write(pub Recipient<WriteResult>, pub usize, pub Vec<u8>);

impl Message for Write {
    type Result = ();
}

#[derive(Clone)]
pub struct WriteResult(pub usize, pub Option<ID>);

impl Message for WriteResult {
    type Result = ();
}

/// delete data message
#[derive(Clone)]
pub struct Drop(pub ID);

impl Message for Drop {
    type Result = ();
}

/// update data message
#[derive(Clone)]
pub struct Swap(pub ID, pub Vec<u8>);

impl Message for Swap {
    type Result = ();
}
