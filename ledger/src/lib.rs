//! The Cardano Ledger
//!
//! The root contains era independent types and utilities such as [`Block`], [`crypto`],
//! [`Unique`], etc. Era dependent types are in their respective modules. Types are defined once in
//! their respective era module, and reused if necessary in newer eras. For example, data for
//! plutus scripts is defined as [`alonzo::script::Data`] and reused in all following eras.

extern crate alloc;

pub mod crypto;
pub mod epoch;
pub mod interval;
pub mod slot;

mod address;
pub use address::Address;

pub mod block;
pub use block::Block;

pub mod transaction;
pub use transaction::Transaction;

mod unique;
pub use unique::Unique;

mod url;
pub use url::Url;

pub mod allegra;
pub mod alonzo;
pub mod babbage;
pub mod byron;
pub mod conway;
pub mod mary;
pub mod shelley;

/// Match on an era-independent enum and run one body for all era variants.
///
/// ### Examples
///
/// ```ignore
/// era_independent!(tx: Transaction, |t| {
///     t.do_stuff()
/// })
/// ```
///
/// For [`Block`], add a second closure that only handles EBB:
///
/// ```ignore
/// era_independent!(
///     block: Block,
///     |b| {
///         b.do_stuff()
///     },
///     |ebb| {
///         ebb.do_ebb_stuff()
///     }
/// )
/// ```
#[macro_export]
macro_rules! era_independent {
	($value:ident : Block, |$b:ident| $body:block, |$ebb:ident| $ebb_body:block) => {
		match $value {
			$crate::Block::Boundary($ebb) => $ebb_body,
			$crate::Block::Byron($b) => $body,
			$crate::Block::Shelley($b) => $body,
			$crate::Block::Allegra($b) => $body,
			$crate::Block::Mary($b) => $body,
			$crate::Block::Alonzo($b) => $body,
			$crate::Block::Babbage($b) => $body,
			$crate::Block::Conway($b) => $body,
		}
	};
	($value:ident : $enum:path, |$b:ident| $body:block) => {
		match $value {
			$enum::Byron($b) => $body,
			$enum::Shelley($b) => $body,
			$enum::Allegra($b) => $body,
			$enum::Mary($b) => $body,
			$enum::Alonzo($b) => $body,
			$enum::Babbage($b) => $body,
			$enum::Conway($b) => $body,
		}
	};
}
