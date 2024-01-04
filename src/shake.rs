use std::{
    io::{Read, Write},
    net::TcpStream,
};

use openssl::{
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};

use crate::bufferable::Bufferable;

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

        buffer.append(&mut u8_group_to_vec(u16_to_u8_group(
            public_key_bytes.len() as u16,
        )));
        buffer.append(&mut public_key_bytes);

        buffer.append(&mut u8_group_to_vec(
            u16_to_u8_group(self.data.len() as u16),
        ));
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

        let mut public_key =
            vec![0; recompute_u16_from_u8_group(public_key_size_u8_group.to_vec()) as usize];
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

        let mut data = vec![0; recompute_u16_from_u8_group(data_size_u8_group.to_vec()) as usize];
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

/// Turns u16 sizes into 3 u8 sizes.
/// The head (1) contains the first values from 0 - 256, after that it no longer increases
/// The multiple (2) contains the amount of times you can multiply times the head
/// The leftover contains the numbers between 0 - 256 that remain to complete the u16 original value
fn u16_to_u8_group(num_u16: u16) -> (u8, u8, u8) {
    let target_size = u16::pow(2, 8) - 1;

    // Always stay between 0 - 255
    let head = u16::min(target_size, num_u16) as u8;
    let multiples = (num_u16 as f32 / target_size as f32).floor() as u8;
    let leftover = (num_u16 % target_size) as u8;

    (head, multiples, leftover)
}

fn recompute_u16_from_u8_group(nums: Vec<u8>) -> u16 {
    let head = *nums.get(0).expect("u8 group head") as u16;
    let multiple = *nums.get(1).expect("u8 multiple head") as u16;
    let leftover = *nums.get(2).expect("u8 group head") as u16;

    head * multiple + leftover
}

/// makes it easier to append to a buffer  
fn u8_group_to_vec((u8_1, u8_2, u8_3): (u8, u8, u8)) -> Vec<u8> {
    vec![u8_1, u8_2, u8_3]
}

#[cfg(test)]
mod tests {
    use openssl::{pkey::PKey, rsa::Rsa};
    use std::{
        io::Write,
        net::{TcpListener, TcpStream},
    };

    use crate::{
        bufferable::Bufferable,
        shake::{recompute_u16_from_u8_group, u16_to_u8_group, u8_group_to_vec, Handshake, Shake},
        stream::Stream,
    };

    #[test]
    fn test_u16_to_u8() {
        assert_eq!(u16_to_u8_group(100), (100, 0, 100));
        assert_eq!(u16_to_u8_group(400), (255, 1, 145));
        assert_eq!(u16_to_u8_group(500), (255, 1, 245));
        assert_eq!(u16_to_u8_group(600), (255, 2, 90));
        assert_eq!(u16_to_u8_group(700), (255, 2, 190));
        assert_eq!(u16_to_u8_group(7000), (255, 27, 115));
    }

    #[test]
    fn test_u8_group_reconstruct() {
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(100))),
            100
        );
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(400))),
            400
        );
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(500))),
            500
        );
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(600))),
            600
        );
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(700))),
            700
        );
        assert_eq!(
            recompute_u16_from_u8_group(u8_group_to_vec(u16_to_u8_group(7000))),
            7000
        );
    }

    #[test]
    fn write_read_and_perform_handshake_stream() {
        let server = TcpListener::bind("127.0.0.1:4645").unwrap();

        let client_tcp_stream = TcpStream::connect("127.0.0.1:4645").unwrap();
        let (server_tcp_stream, _) = server.accept().unwrap();

        let mut client_stream = Stream {
            handshaken: Handshake::UNSHAKEN,
            tcp_stream: client_tcp_stream,
        };

        let mut server_stream = Stream {
            handshaken: Handshake::UNSHAKEN,
            tcp_stream: server_tcp_stream,
        };

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
