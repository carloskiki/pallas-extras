//! Data that constitutes a witness.
//!
//! The Byron era has different witnesses for redeem addresses and verifying key addresses. 

mod redeemer;
pub use redeemer::Redeemer;

mod veryfying_key;
pub use veryfying_key::VerifyingKey;
