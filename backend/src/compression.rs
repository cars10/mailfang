use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

const MAX_DECOMPRESSED_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB

pub fn compress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::with_capacity(8192.min(MAX_DECOMPRESSED_SIZE as usize));
    let mut buf = [0u8; 8192];
    loop {
        let n = decoder.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if decompressed.len().saturating_add(n) > MAX_DECOMPRESSED_SIZE as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "decompression exceeded size limit",
            ));
        }
        decompressed.extend_from_slice(&buf[..n]);
    }
    Ok(decompressed)
}
