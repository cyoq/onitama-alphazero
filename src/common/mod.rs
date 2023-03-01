#[inline]
pub const fn get_bit(x: u32, n: usize) -> u32 {
    x >> (31 - n) & 1
}

/// Convert (u32, u32) to a bitboard representation
/// Assuming that (0, 0) starts at top left corner
/// and (4, 4) is in the bottom right corner
#[inline]
pub const fn from_2d_to_bitboard(value: (u32, u32)) -> u32 {
    let (y, x) = value;
    let z = y * 5 + x;
    // value starting with 1 at most significant bit
    // that will be moved at the certain position
    0x8000_0000 >> z
}

#[inline]
pub const fn from_2d_to_1d(value: (u32, u32)) -> u32 {
    let (y, x) = value;
    return y * 5 + x;
}

#[cfg(test)]
mod tests {
    use crate::common::{from_2d_to_bitboard, get_bit};

    #[test]
    fn test_get_bit() {
        let bits: u32 = 0b0000_1111_0000_1111_0000_1111_0000_1111;
        let expected = vec![
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1,
        ];

        for (i, bit) in expected.iter().enumerate() {
            assert_eq!(get_bit(bits, i), *bit);
        }
    }

    #[test]
    fn test_2d_to_bitboard_conversion_0_0() {
        let value: (u32, u32) = (0, 0);
        // 10000
        // 00000
        // 00000
        // 00000
        // 00000
        // 0000000 - Trailing zeroes
        let expected: u32 = 0x8000_0000;
        let result = from_2d_to_bitboard(value);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_2d_to_bitboard_conversion_2_1() {
        let value: (u32, u32) = (2, 1);
        // 00000
        // 00000
        // 01000
        // 00000
        // 00000
        // 0000000 - Trailing zeroes
        let expected: u32 = 0x0010_0000;
        let result = from_2d_to_bitboard(value);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_2d_to_bitboard_conversion_4_4() {
        let value: (u32, u32) = (4, 4);
        // 00000
        // 00000
        // 00000
        // 00000
        // 00001
        // 0000000 - Trailing zeroes
        let expected: u32 = 0x0000_0080;
        let result = from_2d_to_bitboard(value);
        assert_eq!(result, expected);
    }
}
