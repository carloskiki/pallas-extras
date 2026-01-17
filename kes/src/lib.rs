use signature::Signer;

pub mod single_use;
pub mod sum;

pub use single_use::SingleUse;
pub use sum::Sum;

/// Trait for forward secure key evolution.
pub trait Evolve: Sized {
    /// The number of periods for the key.
    ///
    /// Can be seen the number of times the key can evolve plus 1.
    const PERIOD_COUNT: u32;

    /// Evolve the key to the next period.
    ///
    /// This should always fail when the period has reached `PERIOD_COUNT - 1`. It can also
    /// fail for implementation specific reasons.
    fn evolve(self) -> Option<Self>;

    /// Every time the key evolves, the period is incremented by 1, starting at 0.
    fn period(&self) -> u32;

    /// Sign a message and then evolve the key.
    fn try_sign_evolve<S>(self, msg: &[u8]) -> signature::Result<(Option<Self>, S)>
    where
        Self: Signer<S>,
    {
        let signature = self.try_sign(msg)?;
        let evolution = self.evolve();
        Ok((evolution, signature))
    }
}

/// Also know as KES.
///
/// A signature with a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyEvolvingSignature<S> {
    /// The signature.
    pub signature: S,
    /// The period.
    pub period: u32,
}
