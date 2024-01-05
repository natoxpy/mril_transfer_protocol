// When transfering Large amounts of data

use uuid::Uuid;

use crate::{
    bufferable::Bufferable,
    utils::macros::{u8_bytes_to_usize, usize_to_u8_bytes},
    package::{package_uuid::get_uuid_from_tcp_stream, Package},
};

/// # Packages procedure 
/// Use to allow other structures to turn 
/// their data to packages
pub trait Packable {
    fn pack() -> Packages;
}

/// # Protocol procedure
/// Packages are send one after another.
/// After each package one byte is reserve to let 
/// the other side know whether to expect more or not
/// - 1: more incoming packages
/// - 0: no more incoming packages 
pub struct Packages {
    pub data: Vec<u8>,
}

fn data_to_vec_data(data: Vec<u8>) -> Vec<Vec<u8>> {
    let u8_max = isizex32::pow(2, 8) - 1;
    let mut i = 0;

    data.split(|| {
        i += 1; 

        if i % u8_max {
            true
        } else {
            false
        }
    });
}

impl Bufferable for Packages {
    fn to_buffer(self) -> Vec<u8> {
        let mut bytes = vec![];
        let mut packages = vec![];

        let mut uuid_bytes = self.meta_uuid.as_bytes();

        bytes.append(&mut uuid_bytes);

        bytes.push(0);

        bytes
    }

    fn from_stream(tcp_stream: &mut std::net::TcpStream) -> Self {
         
    }
}
