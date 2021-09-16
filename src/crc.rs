/// `n_bits` の範囲は `1..=8` でなければならない。
pub const fn crc_update(crc_pre: u16, data: u8, n_bits: u8) -> u16 {
    let mut crc = crc_pre ^ ((data as u16) << (16 - n_bits));

    let mut i = 0;
    while i < n_bits {
        let carry = (crc & (1 << 15)) != 0;
        crc <<= 1;
        if carry {
            crc ^= 0x1021;
        }
        i += 1;
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_update() {
        assert_eq!(crc_update(0, 0, 1), 0);
        assert_eq!(crc_update(0, 0, 8), 0);
        assert_eq!(crc_update(0, 1, 1), 0x1021);
        assert_eq!(crc_update(0, 2, 2), 0x2042);
        assert_eq!(crc_update(0, 3, 2), 0x3063);
        assert_eq!(crc_update(crc_update(0, 1, 1), 1, 1), 0x3063);
        assert_eq!(crc_update(crc_update(0, 0xFF, 8), 0xFF, 8), 0x1D0F);
    }
}
