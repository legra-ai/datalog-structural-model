//! Canonical content identities for programs and validation contracts.

mod contract;
mod domain;
mod hex;
mod ir_preimage;
mod logic_preimage;
mod output;
mod sink;

pub use contract::ValidationContract;
pub use hex::DigestParseError;
pub use output::{ContractDigest, ProgramDigest};
