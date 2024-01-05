use std::{
    io::{Read, Write},
    net::TcpStream,
};

use openssl::{
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};

use crate::bufferable::Bufferable;
// use crate::utils::{recompute_u16_from_u8_group, u16_to_u8_group, u8_group_to_vec};
use crate::utils::macros::{u8_bytes_to_usize, usize_to_u8_bytes};

/// A Shake is required to establish a mutually secured encrypted connection
/// with the client and server.
///
/// A shake may include a bit of data of the client and also include the public key of the client.
/// If a HandShake is not initialized between both parties the communication will not be secure.  
///
/// The data can only be a maximum of
#[derive(Debug, Clone)]
pub struct Shake {
    pub data: Vec<u8>,
    pub public_key: PKey<Public>,
}

impl Bufferable for Shake {
    fn to_buffer(mut self) -> Vec<u8> {
        let mut buffer = vec![];
        let mut public_key_bytes = self.public_key.public_key_to_pem().unwrap();

        let mut public_key_bytes_length = usize_to_u8_bytes!(public_key_bytes.len(); 3).to_vec();
        let mut data_length = usize_to_u8_bytes!(self.data.len(); 3).to_vec();

        buffer.append(&mut public_key_bytes_length);
        buffer.append(&mut public_key_bytes);

        buffer.append(&mut data_length);
        buffer.append(&mut self.data);

        buffer
    }

    fn from_stream(stream: &mut TcpStream) -> Self {
        let public_key_pem = Self::read_public_key(stream);
        let public_key =
            PKey::public_key_from_pem(&public_key_pem).expect("expected valid public key pem");
        let data = Self::read_data(stream);

        Self { data, public_key }
    }
}

impl Shake {
    fn read_public_key(stream: &mut TcpStream) -> Vec<u8> {
        let mut public_key_size_u8_group = [0; 3];
        stream
            .read(&mut public_key_size_u8_group)
            .expect("expected to read public key size u8 group");

        let mut public_key = vec![0; u8_bytes_to_usize!(public_key_size_u8_group)];

        stream
            .read(&mut public_key)
            .expect("expected to read public key");
        public_key
    }

    fn read_data(stream: &mut TcpStream) -> Vec<u8> {
        let mut data_size_u8_group = [0; 3];

        stream
            .read(&mut data_size_u8_group)
            .expect("expected to read data size u8 group");

        let mut data = vec![0; u8_bytes_to_usize!(data_size_u8_group)];

        stream.read(&mut data).expect("expected to read data");

        data
    }
}

/// After the shake is received by both parties and is validated it will
/// be recognized and public keys will be store for further use
#[derive(Debug)]
pub enum Handshake {
    /// A secure handshake which includes its corresponding Private key
    /// and the clients public key for secure communications  
    SHAKEN(Shake, Rsa<Private>),

    /// A unsecure method of communication which makes
    /// use of raw data transfer with no encryption
    UNSHAKEN,
}

/// Quick method to perform a simple handshake
pub fn perform_handshake(stream: &mut TcpStream) -> Handshake {
    let key = Rsa::generate(2048).unwrap();
    let public_key = PKey::public_key_from_pem(&key.public_key_to_pem().unwrap()).unwrap();

    let shake = Shake {
        data: String::from("awa").as_bytes().to_vec(),
        public_key,
    };

    stream.write(&shake.to_buffer()).unwrap();

    Handshake::SHAKEN(Shake::from_stream(stream), key)
}

#[cfg(test)]
mod tests {
    use openssl::{pkey::PKey, rsa::Rsa};
    use std::io::Write;

    use crate::{
        bufferable::Bufferable,
        shake::{Handshake, Shake},
    };

    #[test]
    fn write_read_and_perform_handshake_stream() {
        let (mut server_stream, mut client_stream) =
            crate::tests::stablish_server_client_connection().split();

        let client_data = "Hello I'm the client";
        let server_data = "Hello I'm the server";

        let client_key = Rsa::generate(2048).unwrap();
        let server_key = Rsa::generate(2048).unwrap();

        let client_public_key =
            PKey::public_key_from_pem(&client_key.public_key_to_pem().unwrap()).unwrap();
        let server_public_key =
            PKey::public_key_from_pem(&server_key.public_key_to_pem().unwrap()).unwrap();

        let client_shake = Shake {
            data: String::from(client_data).as_bytes().to_vec(),
            public_key: client_public_key.clone(),
        };

        let server_shake = Shake {
            data: String::from(server_data).as_bytes().to_vec(),
            public_key: server_public_key.clone(),
        };

        client_stream
            .tcp_stream
            .write(&client_shake.clone().to_buffer())
            .unwrap();

        server_stream
            .tcp_stream
            .write(&server_shake.clone().to_buffer())
            .unwrap();

        // Data transmitted from client
        let shake_from_client = Shake::from_stream(&mut server_stream.tcp_stream);
        // Data transmitted from server
        let shake_from_server = Shake::from_stream(&mut client_stream.tcp_stream);

        let shake_from_client_data_string =
            String::from_utf8_lossy(&shake_from_client.data).to_string();
        let shake_from_server_data_string =
            String::from_utf8_lossy(&shake_from_server.data).to_string();

        let shake_from_client_public_key = shake_from_client.public_key.clone();
        let shake_from_server_public_key = shake_from_server.public_key.clone();

        client_stream.handshaken = Handshake::SHAKEN(shake_from_server, client_key);
        server_stream.handshaken = Handshake::SHAKEN(shake_from_client, server_key);

        /*
            checks whether the server and client received the
            correct message which they sent to each other.
        */
        assert_eq!(String::from(client_data), shake_from_client_data_string);
        assert_eq!(String::from(server_data), shake_from_server_data_string);

        /*
            checks whether the server and client received the correct message
            which they sent to each other, validating they are accure is important
            to ensure encryption will be perform with the correct keys.
        */
        assert_eq!(
            shake_from_client_public_key.public_key_to_pem().unwrap(),
            client_public_key.public_key_to_pem().unwrap()
        );
        assert_eq!(
            shake_from_server_public_key.public_key_to_pem().unwrap(),
            server_public_key.public_key_to_pem().unwrap()
        );
    }
}
