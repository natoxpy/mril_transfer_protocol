use std::{net::TcpListener, fs};

use mril_transfer_protocol::{package::packages::Packages, shake::Handshake, stream::Stream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3400").expect("Tcp listener");
    println!("Listening on port 3400");
    let mut t = 0;

    loop {
        t += 1;
        let (tcp_stream, _) = listener.accept().expect("Tcp stream");
        // mril_transfer_protocol::stream::Stream::connect_stream(tcp_stream);
        let mut stream = Stream {
            handshaken: Handshake::UNSHAKEN,
            tcp_stream,
        };

        let now = std::time::Instant::now();

        let packages = Packages::read_from(&mut stream.tcp_stream);

        assert_eq!(packages.data.len(), 112591267);

        // println!("[server]: received {:?} bytes", packages.data.len());

        fs::write("music.flac", packages.data).unwrap();
        
        println!("time taken {:?} : {}", now.elapsed(), t);
    }
}

// fn main() {
//     let mut rand = rand::thread_rng();
//     let bytes = [0; 128]
//         .to_vec()
//         .iter()
//         .map(|_| rand.gen())
//         .collect::<Vec<u8>>();

//     let (mut server, mut client) = stablish_server_client_connection().split();

//     println!("\n\nbytes {:?}", bytes);

//     std::thread::spawn(move || {
//         let mut packages = Packages::new(bytes);
//         packages.set_package_size(PackageSize::TINY);

//         packages.listen_reports(|report| {
//             println!("\n\n[client]: {:?}", report);
//         });

//         packages.write_to(&mut client.tcp_stream);
//     });

//     std::thread::spawn(move || {
//         let packages = Packages::read_from(&mut server.tcp_stream);
//         println!("\n\n[server]: {:?}", packages);
//     });
// }
