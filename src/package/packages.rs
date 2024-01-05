// When transfering Large amounts of data

use uuid::Uuid;

pub struct Packages {
    pub meta_uuid: Uuid,

    /// Max data allowed: 2^40 (Approximately 1100GB)
    pub data: Vec<u8>
}