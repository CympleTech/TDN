use std::str::FromStr;

use crate::Error;

const HARDENED_BIT: u32 = 1 << 31;

/// A child number for a derived key
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChildNumber(u32);

impl ChildNumber {
    pub fn is_hardened(&self) -> bool {
        self.0 & HARDENED_BIT == HARDENED_BIT
    }

    pub fn is_normal(&self) -> bool {
        self.0 & HARDENED_BIT == 0
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub fn hardened_from_u32(index: u32) -> Self {
        ChildNumber(index | HARDENED_BIT)
    }

    pub fn non_hardened_from_u32(index: u32) -> Self {
        ChildNumber(index)
    }
}

impl FromStr for ChildNumber {
    type Err = Error;

    fn from_str(child: &str) -> Result<ChildNumber, Error> {
        let (child, mask) = if child.ends_with('\'') {
            (&child[..child.len() - 1], HARDENED_BIT)
        } else {
            (child, 0)
        };

        let index: u32 = child.parse().map_err(|_| Error::InvalidChildNumber)?;

        if index & HARDENED_BIT == 0 {
            Ok(ChildNumber(index | mask))
        } else {
            Err(Error::InvalidChildNumber)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DerivationPath {
    path: Vec<ChildNumber>,
}

impl FromStr for DerivationPath {
    type Err = Error;

    fn from_str(path: &str) -> Result<DerivationPath, Error> {
        let mut path = path.split('/');

        if path.next() != Some("m") {
            return Err(Error::InvalidDerivationPath);
        }

        Ok(DerivationPath {
            path: path
                .map(str::parse)
                .collect::<Result<Vec<ChildNumber>, Error>>()?,
        })
    }
}

impl DerivationPath {
    pub fn as_ref(&self) -> &[ChildNumber] {
        &self.path
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChildNumber> {
        self.path.iter()
    }
}

pub trait IntoDerivationPath {
    fn into(self) -> Result<DerivationPath, Error>;
}

impl IntoDerivationPath for DerivationPath {
    fn into(self) -> Result<DerivationPath, Error> {
        Ok(self)
    }
}

impl IntoDerivationPath for &str {
    fn into(self) -> Result<DerivationPath, Error> {
        self.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_path() {
        let path: DerivationPath = "m/44'/60'/0'/0".parse().unwrap();

        assert_eq!(
            path,
            DerivationPath {
                path: vec![
                    ChildNumber(44 | HARDENED_BIT),
                    ChildNumber(60 | HARDENED_BIT),
                    ChildNumber(0 | HARDENED_BIT),
                    ChildNumber(0),
                ],
            }
        );
    }
}
