use std::net::TcpListener;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:3400").expect("Tcp listener");
    println!("Listening on port 3400");

    loop {
        let (tcp_stream, _) = listener.accept().expect("Tcp stream");
        mril_transfer_protocol::stream::Stream::connect_stream(tcp_stream);
    }
}
