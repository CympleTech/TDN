use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum Error {
    Hex,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
