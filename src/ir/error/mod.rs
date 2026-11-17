//! [`ProgramValidationError`], its stable codes, and its `Display` / `Error`
//! / `From` integration with [`StratificationError`](crate::StratificationError).

mod code;
mod display;
mod variants;

pub use variants::ProgramValidationError;
