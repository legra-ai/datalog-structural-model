//! The shared digest pipeline.

use super::sink::PreimageSink;

/// A value with a canonical, order-preserving preimage.
pub(super) trait CanonicalPreimage {
    /// Writes this value's type-specific fields in their fixed order.
    fn write_preimage(&self, sink: &mut PreimageSink);

    /// Writes a sequence: its length, then each element in authored order.
    fn write_sequence(items: &[Self], sink: &mut PreimageSink)
    where
        Self: Sized,
    {
        sink.count(items.len());
        for item in items {
            item.write_preimage(sink);
        }
    }
}

/// A canonical preimage hashed under its own versioned domain tag.
///
/// Implementors supply only the tag, the output type, and (through
/// [`CanonicalPreimage`]) the fields; the default [`Self::digest`] method
/// carries the shared pipeline: domain tag first, then the fields, then BLAKE3.
pub(super) trait DomainDigest: CanonicalPreimage {
    /// Versioned domain-separation tag. Changing the encoding requires a new tag.
    const DOMAIN: &'static str;

    /// The typed 32-byte identity this domain produces.
    type Output: From<[u8; 32]>;

    /// Computes the identity.
    fn digest(&self) -> Self::Output {
        let mut sink = PreimageSink::new();
        sink.text(Self::DOMAIN);
        self.write_preimage(&mut sink);
        Self::Output::from(sink.finish())
    }
}
