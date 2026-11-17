//! [`ErrorCode`] — stable, kebab-case identifiers for error variants.

use std::fmt;

/// Stable machine-readable identifier of an error variant.
///
/// Codes are kebab-case and never change when a variant is renamed; the
/// census test pins every one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorCode(&'static str);

impl ErrorCode {
    pub(crate) const fn new(code: &'static str) -> Self {
        Self(code)
    }

    /// The kebab-case code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
