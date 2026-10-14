// Reflected CRC polynomials.
//
// CRC-32/ISCSI (Castagnoli):
//   normal polynomial    = 0x1EDC6F41
//   reflected polynomial = 0x82F63B78
//
// CRC-16/ARC:
//   normal polynomial    = 0x8005
//   reflected polynomial = 0xA001
const CRC32_CASTAGNOLI_POLY: u32 = 0x82F6_3B78;
const CRC16_ARC_POLY: u16 = 0xA001;

/// Computes a CRC-32 Castagnoli checksum over `data`.
///
/// This implements the `RevEng` `CRC-32/ISCSI` model, also known as CRC-32C:
///
/// - polynomial: `0x1EDC6F41`
/// - initial value: `0xFFFFFFFF`
/// - input reflected: true
/// - output reflected: true
/// - final XOR: `0xFFFFFFFF`
#[must_use]
pub fn crc32_castagnoli(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF;

    for &byte in data {
        crc ^= u32::from(byte);

        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ CRC32_CASTAGNOLI_POLY
            } else {
                crc >> 1
            };
        }
    }

    crc ^ 0xFFFF_FFFF
}

/// Computes a CRC-16/ARC checksum over `data`.
///
/// `CRC-16/ARC` is the algorithm listed by `RevEng` with the alias `CRC-16`.
///
/// - polynomial: `0x8005`
/// - initial value: `0x0000`
/// - input reflected: true
/// - output reflected: true
/// - final XOR: `0x0000`
#[must_use]
pub fn crc16_arc(data: &[u8]) -> u16 {
    let mut crc = 0x0000;

    for &byte in data {
        crc ^= u16::from(byte);

        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ CRC16_ARC_POLY
            } else {
                crc >> 1
            };
        }
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::assert_bit_exact;

    const CHECK_INPUT: &[u8] = b"123456789";

    // RevEng:
    // reveng -m CRC-32/ISCSI -c 313233343536373839
    #[test]
    fn test_crc32_castagnoli_matches_reveng_oracle() {
        let actual = [crc32_castagnoli(CHECK_INPUT)];
        let expected = [0xE306_9283_u32];

        assert_bit_exact!(actual, expected);
    }

    // RevEng:
    // reveng -m CRC-16/ARC -c 313233343536373839
    #[test]
    fn test_crc16_arc_matches_reveng_oracle() {
        let actual = [crc16_arc(CHECK_INPUT)];
        let expected = [0xBB3D_u16];

        assert_bit_exact!(actual, expected);
    }
}
