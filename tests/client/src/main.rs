use std::net::TcpStream;

fn main() {
    let tcp_stream = TcpStream::connect("127.0.0.1:3400").expect("expected connection");
    mril_transfer_protocol::stream::Stream::connect_stream(tcp_stream);
}
