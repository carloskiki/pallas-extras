use minicbor::{Decode, Encode};
use rand::distr::Alphanumeric;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{borrow::Cow, iter, path::Path};

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct AddressBook<'a> {
    #[n(0)]
    timestamp: u64,
    #[b(1)]
    #[serde(borrow)]
    entries: Vec<Entry<'a>>,
    #[b(2)]
    #[serde(borrow)]
    style: Option<Style<'a>>,
    #[n(3)]
    rating: Option<f64>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Entry<'a> {
    #[b(0)]
    #[serde(borrow)]
    firstname: Cow<'a, str>,
    #[b(1)]
    #[serde(borrow)]
    lastname: Cow<'a, str>,
    #[n(2)]
    birthday: u32,
    #[b(3)]
    #[serde(borrow)]
    addresses: Vec<Address<'a>>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Address<'a> {
    #[b(0)]
    #[serde(borrow)]
    street: Cow<'a, str>,
    #[b(1)]
    #[serde(borrow)]
    houseno: Cow<'a, str>,
    #[n(2)]
    postcode: u32,
    #[b(3)]
    #[serde(borrow)]
    city: Cow<'a, str>,
    #[b(4)]
    #[serde(borrow)]
    country: Cow<'a, str>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
enum Style<'a> {
    #[n(0)]
    Version1,
    #[n(1)]
    Version2,
    #[n(2)]
    Version3(#[n(0)] bool, #[n(1)] u64),
    #[b(3)]
    Version4 {
        #[b(0)]
        #[serde(borrow)]
        path: Cow<'a, str>,
        #[n(1)]
        timestamp: u64,
    },
}

fn bench(label: &str, mut f: impl FnMut() -> bool) {
    const ITERATIONS: u32 = 1000;
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        assert!(f())
    }
    eprintln!("{label:24} {:0.2?}", start.elapsed() / ITERATIONS)
}

#[test]
fn packed_serde_cbor_vs_minicbor() {
    let book = gen_addressbook(16);
    let book_bytes_serde = serde_cbor::ser::to_vec_packed(&book).unwrap();
    let book_bytes_minicbor = minicbor::to_vec(&book).unwrap();

    println!();
    bench("encode/serde_cbor", || {
        serde_cbor::ser::to_vec_packed(&book).is_ok()
    });
    bench("encode/minicbor", || minicbor::to_vec(&book).is_ok());
    bench("decode/serde_cbor", || {
        serde_cbor::from_slice::<AddressBook>(&book_bytes_serde).is_ok()
    });
    bench("decode/minicbor", || {
        minicbor::decode::<AddressBook>(&book_bytes_minicbor).is_ok()
    });
}

fn gen_addressbook(n: usize) -> AddressBook<'static> {
    fn gen_string(g: &mut ThreadRng) -> Cow<'static, str> {
        Cow::Owned(
            iter::repeat_with(|| char::from(g.sample(Alphanumeric)))
                .take(128)
                .collect(),
        )
    }

    fn gen_address(g: &mut ThreadRng) -> Address<'static> {
        Address {
            street: gen_string(g),
            houseno: gen_string(g),
            postcode: g.random(),
            city: gen_string(g),
            country: gen_string(g),
        }
    }

    fn gen_style(g: &mut ThreadRng) -> Option<Style<'static>> {
        let s = match g.random_range(0..5) {
            0 => return None,
            1 => Style::Version1,
            2 => Style::Version2,
            3 => Style::Version3(g.random(), g.random()),
            4 => Style::Version4 {
                path: gen_string(g),
                timestamp: g.random(),
            },
            _ => unreachable!(),
        };
        Some(s)
    }

    fn gen_entry(g: &mut ThreadRng, n: usize) -> Entry<'static> {
        Entry {
            firstname: gen_string(g),
            lastname: gen_string(g),
            birthday: g.random(),
            addresses: {
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(gen_address(g))
                }
                v
            },
        }
    }

    let mut g = rand::rng();

    AddressBook {
        timestamp: g.random(),
        entries: {
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(gen_entry(&mut g, n))
            }
            v
        },
        style: gen_style(&mut g),
        rating: if g.random() {
            Some(g.random_range(-2342.42342..234423.2342))
        } else {
            None
        },
    }
}
