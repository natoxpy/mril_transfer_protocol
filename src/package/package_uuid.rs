use std::{net::TcpStream, io::Read};

use rand::Rng;
use uuid::Uuid;

use crate::utils::macros::usize_to_u8_bytes;

pub mod typemarkers {
    pub const HANDSKAKE: u8 = 1;
    pub const PACKAGE: u8 = 1;
    pub const PACKAGES: u8 = 2;
}

pub mod encryption {
    pub const UNENCRYPTED: u8 = 0;
    pub const ENCRYPTED: u8 = 1;
}

/// 
pub fn new_uuid(
    item_number: usize,
    free_data: Vec<u8>,
    type_marker: u8,
    encrypted: u8,
) -> Uuid {
    let mut bytes = [0; 16];
    let mut rng = rand::thread_rng();

    for i in 0..3 {
        bytes[i] = rng.gen();
    }

    let item_number_bytes = usize_to_u8_bytes!(item_number; 4);
    for (i, val) in (4..7).enumerate() {
        bytes[val] = item_number_bytes[i];
    }

    for (i, val) in (8..(8 + free_data.len())).enumerate() {
        bytes[val] = free_data[i];
    }

    bytes[14] = type_marker;
    bytes[15] = encrypted;

    Uuid::from_bytes(bytes)
}

pub fn get_uuid_from_tcp_stream(tcp_stream: &mut TcpStream) -> Uuid {
    let mut uuid_bytes = [0; 16];

    tcp_stream.read(&mut uuid_bytes).expect("Expected UUID from tcp stream");

    Uuid::from_bytes(uuid_bytes)
}