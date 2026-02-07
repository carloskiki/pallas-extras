// use std::str::FromStr;
// 
// use plutus::{Program, Context, Budget};
// 
// include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cost-model.rs"));
// 
// #[test]
// fn temp() {
//     const UPLC: &str = include_str!("../ifint-mod.uplc");
//     let program = Program::<String>::from_str(UPLC).unwrap().into_de_bruijn().unwrap();
//     let mut context = Context {
//         model: COST_MODEL,
//         budget: Budget {
//             execution: u64::MAX,
//             memory: u64::MAX,
//         },
//     };
//     let result = program.evaluate(&mut context).unwrap();
//     dbg!(result);
// }
