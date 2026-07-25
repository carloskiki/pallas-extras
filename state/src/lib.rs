pub mod byron;

pub trait State {
    type Environment;
    type Signal<'a>;
    type Error<'a>;

    fn transition<'a>(
        &mut self,
        env: &Self::Environment,
        signal: &'a Self::Signal<'a>,
    ) -> Result<(), Self::Error<'a>>;
}
