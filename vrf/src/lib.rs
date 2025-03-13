use digest::{Output, OutputSizeUser};

pub trait Proof<H>
where 
    H: OutputSizeUser,
{
    fn to_hash(&self) -> Output<H>;
}

pub trait Prover<P, H>
where
    P: Proof<H>,
    H: OutputSizeUser,
{
    fn prove(&self, alpha: &[u8]) -> P;
}

pub trait Verifier<P, H>
where
    P: Proof<H>,
    H: OutputSizeUser,
{
    fn verify(&self, alpha: &[u8], proof: P) -> bool;
}
