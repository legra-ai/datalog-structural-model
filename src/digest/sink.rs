//! Length-framed BLAKE3 preimage writer.

/// Receives the canonical preimage of a value as tagged, length-framed fields.
///
/// Every variable-length field carries its length as a `u64` little-endian
/// prefix, and every enum variant writes a one-byte tag first, so two distinct
/// values can never concatenate to the same byte sequence.
pub(crate) struct PreimageSink {
    hasher: blake3::Hasher,
}

impl PreimageSink {
    pub(super) fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Writes a one-byte variant tag.
    pub(super) fn tag(&mut self, tag: u8) {
        self.hasher.update(&[tag]);
    }

    /// Writes a fixed-width unsigned count or number.
    pub(super) fn count(&mut self, value: usize) {
        // usize is at most 64 bits on every supported target.
        let value = u64::try_from(value).unwrap_or(u64::MAX);
        self.hasher.update(&value.to_le_bytes());
    }

    /// Writes a fixed-width 32-bit number.
    pub(super) fn word(&mut self, value: u32) {
        self.hasher.update(&value.to_le_bytes());
    }

    /// Writes a length-prefixed byte field.
    pub(super) fn frame(&mut self, bytes: &[u8]) {
        self.count(bytes.len());
        self.hasher.update(bytes);
    }

    /// Writes a length-prefixed UTF-8 text field.
    pub(super) fn text(&mut self, text: &str) {
        self.frame(text.as_bytes());
    }

    pub(super) fn finish(self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }
}
