use arbitrary::Arbitrary;
use minicbor::{to_vec, CborLen, Decode, Encode};

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(array)]
enum SampleArrayEncoding<T> {
    #[n(0)]
    Unit,
    #[n(1)]
    Struct {
        #[n(0)]
        field1: String,
        #[n(1)]
        field2: bool,
    },
    #[n(2)]
    TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)]
    Generic(#[n(0)] T),
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(map)]
enum SampleMapEncoding<T> {
    #[n(0)]
    Unit,
    #[n(1)]
    Struct {
        #[n(0)]
        field1: String,
        #[n(1)]
        field2: bool,
    },
    #[n(2)]
    TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)]
    Generic(#[n(0)] T),
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(array)]
struct BytesArrayEncoding {
    #[cbor(n(0), with = "minicbor::bytes")]
    array: [u8; 32],
    #[cbor(n(1), with = "minicbor::bytes")]
    vector: Vec<u8>,
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(map)]
struct BytesMapEncoding {
    #[cbor(n(0), with = "minicbor::bytes")]
    array: [u8; 32],
    #[cbor(n(1), with = "minicbor::bytes")]
    vector: Vec<u8>,
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(transparent)]
struct TransparentEncoding(#[cbor(n(0))] u8);

impl Arbitrary<'_> for SampleArrayEncoding<BytesArrayEncoding> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(match u.choose(&[0, 1, 2, 3])? {
            0 => SampleArrayEncoding::Unit,
            1 => SampleArrayEncoding::Struct {
                field1: Arbitrary::arbitrary(u)?,
                field2: Arbitrary::arbitrary(u)?,
            },
            2 => SampleArrayEncoding::TupleStruct(Arbitrary::arbitrary(u)?, Arbitrary::arbitrary(u)?),
            _ => SampleArrayEncoding::Generic(Arbitrary::arbitrary(u)?),
        })
    }
}

impl Arbitrary<'_> for SampleArrayEncoding<BytesMapEncoding> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(match u.choose(&[0, 1, 2, 3])? {
            0 => SampleArrayEncoding::Unit,
            1 => SampleArrayEncoding::Struct {
                field1: Arbitrary::arbitrary(u)?,
                field2: Arbitrary::arbitrary(u)?,
            },
            2 => SampleArrayEncoding::TupleStruct(Arbitrary::arbitrary(u)?, Arbitrary::arbitrary(u)?),
            _ => SampleArrayEncoding::Generic(Arbitrary::arbitrary(u)?),
        })
    }
}

impl Arbitrary<'_> for SampleMapEncoding<BytesArrayEncoding> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(match u.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleMapEncoding::Unit,
            1 => SampleMapEncoding::Struct {
                field1: Arbitrary::arbitrary(u)?,
                field2: Arbitrary::arbitrary(u)?,
            },
            2 => SampleMapEncoding::TupleStruct(Arbitrary::arbitrary(u)?, Arbitrary::arbitrary(u)?),
            _ => SampleMapEncoding::Generic(Arbitrary::arbitrary(u)?),
        })
    }
}

impl Arbitrary<'_> for SampleMapEncoding<BytesMapEncoding> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(match u.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleMapEncoding::Unit,
            1 => SampleMapEncoding::Struct {
                field1: Arbitrary::arbitrary(u)?,
                field2: Arbitrary::arbitrary(u)?,
            },
            2 => SampleMapEncoding::TupleStruct(Arbitrary::arbitrary(u)?, Arbitrary::arbitrary(u)?),
            _ => SampleMapEncoding::Generic(Arbitrary::arbitrary(u)?),
        })
    }
}

impl Arbitrary<'_> for BytesArrayEncoding {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(BytesArrayEncoding {
            array: [1; 32],
            vector: Arbitrary::arbitrary(u)?,
        })
    }
}

impl Arbitrary<'_> for BytesMapEncoding {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(BytesMapEncoding {
            array: [1; 32],
            vector: Arbitrary::arbitrary(u)?,
        })
    }
}

impl Arbitrary<'_> for TransparentEncoding {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(TransparentEncoding(Arbitrary::arbitrary(u)?))
    }
}

#[test]
fn fuzz() {
    let mut data = [0u8; 1024];
    rand::fill(&mut data);
    let mut ctx = ();

    for _ in 0..100 {
        let mut u = arbitrary::Unstructured::new(&data);
        let val: SampleArrayEncoding<BytesArrayEncoding> = Arbitrary::arbitrary(&mut u).unwrap();
        assert_eq!(val.cbor_len(&mut ctx), to_vec(val).unwrap().len());
        rand::fill(&mut data);
    }

    for _ in 0..100 {
        let mut u = arbitrary::Unstructured::new(&data);
        let val: SampleArrayEncoding<BytesMapEncoding> = Arbitrary::arbitrary(&mut u).unwrap();
        assert_eq!(val.cbor_len(&mut ctx), to_vec(val).unwrap().len());
        rand::fill(&mut data);
    }

    for _ in 0..100 {
        let mut u = arbitrary::Unstructured::new(&data);
        let val: SampleMapEncoding<BytesMapEncoding> = Arbitrary::arbitrary(&mut u).unwrap();
        assert_eq!(val.cbor_len(&mut ctx), to_vec(val).unwrap().len());
        rand::fill(&mut data);
    }

    for _ in 0..100 {
        let mut u = arbitrary::Unstructured::new(&data);
        let val: SampleMapEncoding<BytesArrayEncoding> = Arbitrary::arbitrary(&mut u).unwrap();
        assert_eq!(val.cbor_len(&mut ctx), to_vec(val).unwrap().len());
        rand::fill(&mut data);
    }

    for _ in 0..100 {
        let mut u = arbitrary::Unstructured::new(&data);
        let val: TransparentEncoding = Arbitrary::arbitrary(&mut u).unwrap();
        assert_eq!(val.cbor_len(&mut ctx), to_vec(val).unwrap().len());
        rand::fill(&mut data);
    }
}
