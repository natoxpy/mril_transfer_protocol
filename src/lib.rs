pub mod package;
pub mod shake;
pub mod stream;

// pub mod mtp_incoming;
// pub mod mtp_stream;
// pub mod transmittable;

// pub mod prelude {
//     pub use crate::transmittable::Transmittable;
//     pub use crate::mtp_stream::MtpStream;
// }

#[cfg(test)]
pub mod tests {
    use openssl::rsa::Rsa;

    #[test] 
    fn testa() {
        let rsa = Rsa::generate(2048).unwrap();

        let p = rsa.public_key_to_pem().unwrap();

        println!("{:#?}", p.len());
    }
}