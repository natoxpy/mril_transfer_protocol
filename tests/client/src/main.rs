use mril_transfer_protocol::{
    package::{
        packages::{PackageReportSpeed, Packages, PackagesBatchSize},
        PackageSize,
    },
    shake::Handshake,
    stream::Stream,
};
use std::{fs, net::TcpStream};

// fn main() {
//     let tcp_stream = TcpStream::connect("127.0.0.1:3400").expect("expected connection");
//     let mut stream = Stream {
//         handshaken: Handshake::UNSHAKEN,
//         tcp_stream,
//     };

//     let mut rand = rand::thread_rng();
//     let bytes = vec![0; 100_000_000]
//         .to_vec()
//         .iter()
//         .map(|_| rand.gen())
//         .collect::<Vec<u8>>();

//     println!("\n\ngenerated {:?} bytes to send", bytes.len());
//     let mut packages = Packages::new(bytes);
//     packages.set_package_size(PackageSize::TINY);
//     packages.set_report_speed(PackageReportSpeed::STEADY);
//     packages.set_batch_size(PackagesBatchSize::TINY);

//     packages.listen_reports(|report| {
//         println!("\n\n[client]: {:?}", report);
//     });

//     packages.write_to(&mut stream.tcp_stream);
// }

fn main() {
    let tcp_stream = TcpStream::connect("127.0.0.1:3400").expect("expected connection");
    let mut stream = Stream {
        handshaken: Handshake::UNSHAKEN,
        tcp_stream,
    };

    let mut a = 100;
    loop {
        if a > 100 {
            break;
        }
        a += 1;
    
        let bytes = fs::read("./music_file.flac").unwrap();
        let mut packages = Packages::new(bytes);
    
        packages.set_package_size(PackageSize::LARGE);
        packages.set_report_speed(PackageReportSpeed::FASTEST);
        packages.set_batch_size(PackagesBatchSize::LARGE);
    
        // packages.listen_reports(|report| {
        //     // println!("\n\n[client]: {:?}", report);
        // });
    
        packages.write_to(&mut stream.tcp_stream);
    }
}
