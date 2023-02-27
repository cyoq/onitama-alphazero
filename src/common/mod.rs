#[inline]
pub const fn get_bit(x: u32, n: usize) -> u32 {
    x >> (31 - n) & 1
}
