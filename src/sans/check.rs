//! Helper for computing cyclic redundancy checks.

/// Accumulate a slice of bytes into a cyclic redundancy check value.
pub fn compute_crc(init: u16, r: &[u8]) -> u16 {
    r.iter().fold(init, |acc, b| crc_byte(acc, *b))
}

/// Accumulate a single byte into a cyclic redundancy check value.
fn crc_byte(mut crc: u16, b: u8) -> u16 {
    const CRC_TABLE: [u16; 16] = [
        0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800,
        0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
    ];

    let tmp = CRC_TABLE[(crc & 0xF) as usize];
    crc = (crc >> 4) & 0x0FFF;
    crc = crc ^ tmp ^ CRC_TABLE[(b & 0xF) as usize];

    let tmp = CRC_TABLE[(crc & 0xF) as usize];
    crc = (crc >> 4) & 0x0FFF;
    crc = crc ^ tmp ^ CRC_TABLE[((b >> 4) & 0xF) as usize];

    crc
}
