use minicbor_derive::Encode;

#[derive(Encode)]
enum Test {
    #[n(0)]
    A {
        #[cbor(skip, n = 0)]
        x: i32,
        #[n(1)]
        y: i32,
    },
    #[n(1)]
    B(#[n(0)] i32, #[n(1)] i32),
}

fn main() {}
