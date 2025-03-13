use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use blake2::Blake2bVarCore;
use ponk::network::{
    handshake::{self, VersionTable}, Header, NetworkMagic
};
use digest::{consts::U64, OutputSizeUser};

use minicbor::{Decode, Encode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name1 = std::any::type_name::<<Blake2bVarCore as OutputSizeUser>::OutputSize>();
    let name2 = std::any::type_name::<U64>();
    println!("{} == {}", name1, name2);
    Ok(())
}
