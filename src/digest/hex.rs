//! Lowercase hexadecimal text for 32-byte digests.

use std::fmt;

const DIGEST_LEN: usize = 32;
const HEX_LEN: usize = DIGEST_LEN * 2;
const ALPHABET: &[u8; 16] = b"0123456789abcdef";

/// Why a digest text could not be parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestParseError {
    /// The text is not exactly 64 characters long.
    Length {
        /// Number of bytes found.
        found: usize,
    },
    /// A character is not a lowercase hexadecimal digit.
    Digit {
        /// Byte offset of the offending character.
        offset: usize,
    },
}

impl fmt::Display for DigestParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { found } => {
                write!(f, "digest must be {HEX_LEN} hex characters, found {found}")
            }
            Self::Digit { offset } => {
                write!(
                    f,
                    "digest has a non-lowercase-hex character at offset {offset}"
                )
            }
        }
    }
}

impl std::error::Error for DigestParseError {}

/// Raw digest bytes with hex text conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct DigestBytes(pub(super) [u8; DIGEST_LEN]);

impl DigestBytes {
    pub(super) fn parse(text: &str) -> Result<Self, DigestParseError> {
        let bytes = text.as_bytes();
        if bytes.len() != HEX_LEN {
            return Err(DigestParseError::Length { found: bytes.len() });
        }
        let mut out = [0_u8; DIGEST_LEN];
        for (index, slot) in out.iter_mut().enumerate() {
            let high = Self::nibble(bytes, index * 2)?;
            let low = Self::nibble(bytes, index * 2 + 1)?;
            *slot = (high << 4) | low;
        }
        Ok(Self(out))
    }

    fn nibble(bytes: &[u8], offset: usize) -> Result<u8, DigestParseError> {
        match bytes[offset] {
            digit @ b'0'..=b'9' => Ok(digit - b'0'),
            digit @ b'a'..=b'f' => Ok(digit - b'a' + 10),
            _ => Err(DigestParseError::Digit { offset }),
        }
    }
}

impl fmt::Display for DigestBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            let pair = [
                ALPHABET[usize::from(byte >> 4)],
                ALPHABET[usize::from(byte & 0x0f)],
            ];
            f.write_str(std::str::from_utf8(&pair).map_err(|_| fmt::Error)?)?;
        }
        Ok(())
    }
}
