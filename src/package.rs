
use uuid::Uuid;

use crate::bufferable::Bufferable;

/// pieces of data which can contain upto 16384 bytes of data
#[derive(Debug, Clone)]
pub struct Package {    
    pub uuid: Uuid,
    pub data: Vec<u8>
}

impl Package {

}

impl Bufferable for Package {
    fn from_stream(tcp_stream: &mut std::net::TcpStream) -> Self {

    }

    fn to_buffer(self) -> Vec<u8> {
        
    }
}

/// A collection of packages which is ensured to be received 
/// in the same order in which it was sent
pub struct Packages {
    pub packages: Vec<Package>
}

/// Sent back to the package sender to inform which package was 
/// received and how much data was read from it
pub struct PackageArrival {
    pub uuid: Uuid,
    pub data_length: usize,
    pub was_secured: bool
}
