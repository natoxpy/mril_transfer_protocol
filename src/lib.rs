pub mod shake;
pub mod stream;
pub mod bufferable;
pub mod utils;
pub mod package;

// pub mod mtp_incoming;
// pub mod mtp_stream;
// pub mod transmittable;

// pub mod prelude {
//     pub use crate::transmittable::Transmittable;
//     pub use crate::mtp_stream::MtpStream;
// }


// For method used widely
pub mod tests {
    use std::net::{TcpListener, TcpStream};

    use crate::{stream::Stream, shake::Handshake};


    pub struct Connected {
        pub server: Stream,
        pub client: Stream
    }

    impl Connected {
        /// (Server, Client)
        pub fn split(self) -> (Stream, Stream) {
            (self.server, self.client)
        }
    }

    pub fn stablish_server_client_connection() -> Connected {
        let server = TcpListener::bind("127.0.0.1:4645").unwrap();

        let client_tcp_stream = TcpStream::connect("127.0.0.1:4645").unwrap();
        let (server_tcp_stream, _) = server.accept().unwrap();

        let client_stream = Stream {
            handshaken: Handshake::UNSHAKEN,
            tcp_stream: client_tcp_stream,
        };

        let server_stream = Stream {
            handshaken: Handshake::UNSHAKEN,
            tcp_stream: server_tcp_stream,
        };

        Connected {
            server: server_stream,
            client: client_stream
        }
    }
}