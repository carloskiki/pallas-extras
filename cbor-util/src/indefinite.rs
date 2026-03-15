use tinycbor::{CborLen, Decode, Encode, container, primitive};

#[repr(transparent)]
#[derive(ref_cast::RefCast)]
pub struct Indefinite<T>(pub T);

impl<T> Indefinite<T> {
    pub fn into(self) -> T {
        self.0
    }
}

impl<'a, T> From<&'a T> for &'a Indefinite<T> {
    fn from(value: &'a T) -> Self {
        use ref_cast::RefCast;
        Indefinite::ref_cast(value)
    }
}

impl<T: Encode> Encode for Indefinite<Vec<T>> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.begin_array()?;
        for item in self.0.iter() {
            item.encode(e)?;
        }
        e.end()
    }
}

impl<T: CborLen> CborLen for Indefinite<Vec<T>> {
    fn cbor_len(&self) -> usize {
        2 + self.0.iter().map(|item| item.cbor_len()).sum::<usize>()
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Indefinite<Vec<T>> {
    type Error = container::Error<<T as Decode<'a>>::Error>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor()?;
        let mut items = Vec::new();
        if visitor.definite() {
            return Err(container::Error::Malformed(primitive::Error::InvalidHeader));
        }
        while let Some(item) = visitor.visit() {
            let item = item.map_err(container::Error::Content)?;
            items.push(item);
        }
        Ok(Self(items))
    }
}
