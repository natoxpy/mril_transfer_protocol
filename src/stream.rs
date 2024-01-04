use std::net::{TcpStream, ToSocketAddrs};

use crate::shake::{perform_handshake, Handshake};


#[derive(Debug)]
pub struct Stream {
    pub tcp_stream: TcpStream,
    pub handshaken: Handshake,
}

impl Stream {
    pub fn connect_stream(mut tcp_stream: TcpStream) -> Self {
        let handshaken = perform_handshake(&mut tcp_stream);

        Self {
            tcp_stream,
            handshaken,
        }
    }

    pub fn connect<T: ToSocketAddrs>(addr: T) -> Self {
        let mut tcp_stream =
            TcpStream::connect(addr).expect("Failed to connect to tcp socket address");

        let handshaken = perform_handshake(&mut tcp_stream);

        Self {
            tcp_stream,
            handshaken
        }
    }
}
