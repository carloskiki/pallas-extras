use tinycbor::Decode;

pub trait Inspector<T> {
    fn inspect(value: T) -> T;
}

pub struct Inspect<T, F>(pub T, core::marker::PhantomData<F>);

impl<T, F> Inspect<T, F> {
    pub fn into(self) -> T {
        self.0
    }
}

impl<'a, T: Decode<'a>, F: Inspector<T>> Decode<'a> for Inspect<T, F> {
    type Error = T::Error;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let value = F::inspect(T::decode(d)?);
        Ok(Inspect(value, core::marker::PhantomData))
    }
}
