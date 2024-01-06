// When transfering Large amounts of data

use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    bufferable::Bufferable,
    package::{Package, PackageSize},
};

use super::package_uuid::{encryption, new_uuid, typemarkers};

type PackageReportCallback = fn(PackagesReport);

/// # Packages procedure
/// Use to allow other structures to turn
/// their data to packages
pub trait Packable {
    fn pack() -> Packages;
}

#[derive(Debug)]
pub struct PackagesReport {
    pub sent: usize,
    pub total: usize,
    pub bytes_sent: usize,
    pub total_bytes: usize,
}

/// Slows down reports by skipping reporting certain ones
/// - fastest: No skips
/// - fast: Skips every 1%
/// - steady: Skips every 5%
/// - slow: Skips every 10%
/// - slowest: FAST: Skips every 15%
///
/// Note: If the total packages is too small it will not be noticable  
#[derive(Debug, Default)]
pub enum PackageReportSpeed {
    FASTEST,
    FAST,
    #[default]
    STEADY,
    SLOW,
    SLOWEST,
}

impl PackageReportSpeed {
    pub fn apply_multiplier(&self, value: usize) -> usize {
        match self {
            Self::FASTEST => 1,
            Self::FAST => (value as f32 * 0.01).ceil() as usize,
            Self::STEADY => (value as f32 * 0.05).ceil() as usize,
            Self::SLOW => (value as f32 * 0.1).ceil() as usize,
            Self::SLOWEST => (value as f32 * 0.15).ceil() as usize,
        }
    }
}

/// # Protocol procedure
/// Packages are send one after another.
/// After each package one byte is reserve to let
/// the other side know whether to expect more or not
/// - 1: more incoming packages
/// - 0: no more incoming packages
#[derive(Debug)]
pub struct Packages {
    pub data: Vec<u8>,
    reports_callback: Option<PackageReportCallback>,
    reports_speed: Option<PackageReportSpeed>,
    packages_size: PackageSize,
    batch_size: PackagesBatchSize,
}

/// How many packages are sent at once before the
/// other side is required to sent a response
/// - tiny: 1 package
/// - small: 4 package
/// - medium: 16 packages
/// - large: 64 packages
/// - max: 255 packages

#[derive(Debug, Clone, Copy, Default)]
pub enum PackagesBatchSize {
    TINY,
    SMALL,
    #[default]
    MEDIUM,
    LARGE,
    MAX,
}

impl PackagesBatchSize {
    pub fn to_value(&self) -> u8 {
        match &self {
            Self::TINY => u8::pow(2, 0),
            Self::SMALL => u8::pow(2, 2),
            Self::MEDIUM => u8::pow(2, 4),
            Self::LARGE => u8::pow(2, 6),
            Self::MAX => (u16::pow(2, 8) - 1) as u8,
        }
    }

    pub fn from_value(value: u8) -> Result<Self, ()> {
        match value {
            1 => Ok(Self::TINY),
            4 => Ok(Self::SMALL),
            16 => Ok(Self::MEDIUM),
            64 => Ok(Self::LARGE),
            255 => Ok(Self::MAX),
            _ => Err(()),
        }
    }
}

impl Packages {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            reports_callback: None,
            reports_speed: Some(PackageReportSpeed::default()),
            packages_size: PackageSize::default(),
            batch_size: PackagesBatchSize::default(),
        }
    }

    pub fn set_batch_size(&mut self, batch_size: PackagesBatchSize) {
        self.batch_size = batch_size;
    }

    pub fn set_package_size(&mut self, package_size: PackageSize) {
        self.packages_size = package_size;
    }

    pub fn listen_reports(&mut self, f: PackageReportCallback) {
        self.reports_callback = Some(f)
    }

    pub fn set_report_speed(&mut self, f: PackageReportSpeed) {
        self.reports_speed = Some(f)
    }

    /// # Packages streaming protocol
    /// - A -> B `[size][data]` `[BYTE]`
    ///     - Final byte contains whether more data is incoming or not [0: Not, 1: Yes]
    /// - B -> A `[BYTE]`
    ///     - Response byte says whether it can continue or not
    ///     - [0] continue
    ///     - [1] stop
    pub fn write_to(self, tcp_stream: &mut TcpStream) {
        let data_length = self.data.len();
        let data_vec = data_to_vec_data(self.data, self.packages_size.get_value());
        let packages = data_to_packages(data_vec);

        let mut sent = 0;
        let mut bytes_sent = 0;

        let mut skips_report_count = 0;
        let mut skip_report = 1;

        if self.reports_speed.is_some() {
            skip_report = self.reports_speed.unwrap().apply_multiplier(packages.len());
        }

        let mut batch_count: usize = 0;

        tcp_stream
            .write(&[self.batch_size.to_value()])
            .expect("Expected to write batch size");

        for (i, package) in packages.iter().enumerate() {
            batch_count += 1;
            skips_report_count += 1;
            bytes_sent += package.data.len();

            let mut buffer = package.clone().to_buffer();

            if i == packages.len() {
                buffer.push(0);
            } else {
                buffer.push(1);
            }

            tcp_stream
                .write(&buffer)
                .expect("Expected to be able to write to stream");

            // println!("batch track {}", batch_count % self.batch_size.to_value() as usize);
            // if batch_count % self.batch_size.to_value() as usize == 0 {
            //     println!(
            //         "Writing Batch Size affirmation : {} {} - {}",
            //         batch_count,
            //         self.batch_size.to_value() as usize,
            //         batch_count % self.batch_size.to_value() as usize
            //     );
            //     let mut response = [0; 1];
            //     tcp_stream
            //         .read(&mut response)
            //         .expect("Expected to read response");
            //     // println!("Server response {:?}", response);
            // }

            sent += 1;

            if skips_report_count % skip_report == 0 {
                if self.reports_callback.is_some() {
                    self.reports_callback.unwrap()(PackagesReport {
                        bytes_sent,
                        sent,
                        total: packages.len(),
                        total_bytes: data_length,
                    });
                }
            }
        }

        println!("pre exit response ");
        // let mut response = [0; 1];
        // tcp_stream
        //     .read(&mut response)
        //     .expect("Expected to read response");

        println!("exited");
        if self.reports_callback.is_some() {
            self.reports_callback.unwrap()(PackagesReport {
                bytes_sent,
                sent,
                total: packages.len(),
                total_bytes: data_length,
            });
        }
    }

    pub fn read_from(tcp_stream: &mut TcpStream) -> Self {
        let mut packages = Self::new(vec![]);

        let mut batch_size_byte = [0; 1];
        tcp_stream
            .read(&mut batch_size_byte)
            .expect("Expected to read batch size byte");

        let batch_size = PackagesBatchSize::from_value(batch_size_byte[0])
            .expect("Expected valid number for batch size");
        let mut batch_count: usize = 0;

        loop {
            let mut package = Package::from_stream(tcp_stream);
            batch_count += 1;

            let data_length = package.data.len();

            packages.data.append(&mut package.data);

            let mut footer_byte = [0; 1];

            tcp_stream
                .read(&mut footer_byte)
                .expect("Expected to be able to read from stream");

            println!(
                "- Regular : {} {} - {} : {} / {}",
                batch_count,
                batch_size.to_value() as usize,
                batch_count % batch_size.to_value() as usize,
                data_length,
                packages.data.len()
            );
            // if batch_count % batch_size.to_value() as usize == 0 {
            //     // println!("Batch counted {}", batch_count);
            //     println!(
            //         "Writing Batch Size affirmation : {} {} - {}",
            //         batch_count,
            //         batch_size.to_value() as usize,
            //         batch_count % batch_size.to_value() as usize
            //     );
            //     tcp_stream
            //         .write(&[0])
            //         .expect("Expected to be able to write to stream");
            // }

            // println!("\n\nfooter byte {:?}", footer_byte);
            if footer_byte[0] == 0 {
                break;
            }
        }

        tcp_stream.write(&[0]).unwrap();

        packages
    }
}

fn data_to_vec_data(data: Vec<u8>, max_package_size: usize) -> Vec<Vec<u8>> {
    let mut i = 0;

    data.split_inclusive(|_| {
        i += 1;
        i % max_package_size == 0
    })
    .collect::<Vec<&[u8]>>()
    .iter()
    .map(|i| (**i).to_vec())
    .collect::<Vec<Vec<u8>>>()
}

fn data_to_packages(bytes: Vec<Vec<u8>>) -> Vec<Package> {
    bytes
        .iter()
        .enumerate()
        .map(|(i, package_bytes)| {
            let meta_uuid = new_uuid(i + 1, vec![], typemarkers::PACKAGE, encryption::UNENCRYPTED);

            Package {
                data: package_bytes.clone(),
                meta_uuid,
            }
        })
        .collect::<Vec<Package>>()
}
