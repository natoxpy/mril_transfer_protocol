pub mod package_uuid;
pub mod packages;

use std::io::Read;

use uuid::Uuid;

use crate::{
    bufferable::Bufferable,
    utils::{macros::u8_bytes_to_usize, macros::usize_to_u8_bytes},
};

/// pieces of data which can contain upto 65535 (0.5MB) bytes of data
/// 
#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    /// UUID Explains
    /// - First 4 bytes (0 - 3); Item identifier
    /// - Next 4 bytes (4 - 7); item number (Ensures order will be maintain once received)
    /// - Next 6 bytes (8 - 13); Free bytes
    /// - Before last (14); type marker; `0 -> handshake, 1 -> Package`
    /// - Last byte (15); `encrypted -> 1 / not encrypted -> 0` mark
    pub meta_uuid: Uuid,
    /// Max data allowed; 2^16 - 1
    pub data: Vec<u8>,
}

impl Package {
    pub fn new(data: Vec<u8>, meta_uuid: Uuid) -> Self {
        Self { meta_uuid, data }
    }

    fn read_meta_uuid(tcp_stream: &mut std::net::TcpStream) -> Uuid {
        let mut uuid_bytes = [0; 16];

        tcp_stream
            .read(&mut uuid_bytes)
            .expect("Expected bytes for package UUID");

        Uuid::from_bytes(uuid_bytes)
    }

    fn read_data(tcp_stream: &mut std::net::TcpStream) -> Vec<u8> {
        let mut data_length_bytes = [0; 3];
        tcp_stream
            .read(&mut data_length_bytes)
            .expect("Expected package data length");

        let data_length = u8_bytes_to_usize!(data_length_bytes);

        let mut data_bytes = vec![0; data_length];
        tcp_stream
            .read(&mut data_bytes)
            .expect("Expected data bytes");

        data_bytes
    }
}

impl Bufferable for Package {
    /// Buffer model
    /// - First 16 bytes (0, 15) META UUID
    /// - Next 3 bytes (16, 18) Data length
    /// - Rest data bytes
    fn to_buffer(mut self) -> Vec<u8> {
        let mut buffer = vec![];

        buffer.append(&mut self.meta_uuid.as_bytes().to_vec());

        buffer.append(&mut usize_to_u8_bytes!((self.data.len()); 3).to_vec());

        buffer.append(&mut self.data);

        buffer
    }

    fn from_stream(tcp_stream: &mut std::net::TcpStream) -> Self {
        let meta_uuid = Self::read_meta_uuid(tcp_stream);
        let data = Self::read_data(tcp_stream);

        Self { data, meta_uuid }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        bufferable::Bufferable,
        package::{
            package_uuid::{encryption, new_uuid, typemarkers},
            Package,
        },
    };

    #[test]
    fn transfer_package() {
        let (mut server_stream, mut client_stream) =
            crate::tests::stablish_server_client_connection().split();

        // Data setup by the Client to send to server
        let data = "Hello I'm the client, and this is a friendly package";
        let meta_uuid = new_uuid(1, vec![0], typemarkers::PACKAGE, encryption::UNENCRYPTED);
        let package = Package::new(data.as_bytes().to_vec(), meta_uuid.clone());

        client_stream
            .tcp_stream
            .write(&package.clone().to_buffer())
            .expect("expected to write data to the server");

        // Read in server
        let client_package = Package::from_stream(&mut server_stream.tcp_stream);

        assert_eq!(package.meta_uuid, client_package.meta_uuid);
        assert_eq!(package.data, client_package.data);
    }

    #[test]
    fn test_vec_split() {
        let max_val = 10;

        let data = b"Hello I believe that this will split properly and I will be able to implement the proper person";

        
    }
}
