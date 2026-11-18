//! Typed digest outputs.

use std::{fmt, str::FromStr};

use super::hex::{DigestBytes, DigestParseError};

macro_rules! digest_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(DigestBytes);

        impl $name {
            /// The raw 32 BLAKE3 output bytes.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0.0
            }
        }

        impl From<[u8; 32]> for $name {
            fn from(bytes: [u8; 32]) -> Self {
                Self(DigestBytes(bytes))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = DigestParseError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                DigestBytes::parse(text).map(Self)
            }
        }
    };
}

digest_newtype!(
    /// Canonical content identity of an authored [`crate::Program`].
    ///
    /// Covers rules, body literals and arguments in authored order and every
    /// term, filter, expression and aggregate kind. Displays as lowercase hex.
    ProgramDigest
);

digest_newtype!(
    /// Identity of a [`crate::Program`] checked under one
    /// [`crate::ValidationContract`]: equality mode, limits and termination law.
    ContractDigest
);
