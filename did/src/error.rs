use core::fmt;

/// The BIP-0039 error.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Mnemonic only support 12/15/18/21/24 words.
    BadWordCount(usize),
    /// Entropy was not a multiple of 32 bits or between 128-256n bits in length.
    BadEntropyBitCount(usize),
    /// Mnemonic contains an unknown word.
    UnknownWord(String),
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// Error from libsecp256k1.
    #[cfg(feature = "secp256k1")]
    Secp256k1(secp256k1::Error),
    /// Error for Ed25519
    Ed25519(String),
    /// Invalid Children number.
    InvalidChildNumber,
    /// Invalid derivation path.
    InvalidDerivationPath,
    /// Invalid Extended Private Key.
    InvalidExtendedPrivKey,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadWordCount(count) => write!(
                f,
                "BIP-0039 mnemonic only supports 12/15/18/21/24 words: {}",
                count
            ),
            Error::BadEntropyBitCount(count) => write!(
                f,
                "entropy was not between 128-256 bits or not a multiple of 32 bits: {} bits",
                count
            ),
            Error::UnknownWord(word) => write!(f, "mnemonic contains an unknown word: {}", word),
            Error::InvalidChecksum => write!(f, "mnemonic has an invalid checksum"),
            #[cfg(feature = "secp256k1")]
            Error::Secp256k1(e) => write!(f, "secp256k1: {}", e),
            Error::Ed25519(e) => write!(f, "ed25519: {}.", e),
            Error::InvalidChildNumber => write!(f, "invalid Children number."),
            Error::InvalidDerivationPath => write!(f, "invalid derivation path."),
            Error::InvalidExtendedPrivKey => write!(f, "Invalid Extended Private Key."),
        }
    }
}

impl std::error::Error for Error {}
