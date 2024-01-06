use std::net::TcpStream;

pub trait Bufferable {
    fn to_buffer(self) -> Vec<u8>;
    fn from_stream(tcp_stream: &mut TcpStream) -> Self;
}
