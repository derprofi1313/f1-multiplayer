pub const FNV64_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
pub const FNV64_PRIME: u64 = 0x0000_0100_0000_01b3;
pub const FNV64_PRIME_INV: u64 = 0xce96_5057_aff6_957b;

pub fn fnv1a64(data: impl AsRef<[u8]>) -> u64 {
    fnv1a64_from(FNV64_BASIS, data)
}

pub fn fnv1a64_from(mut h: u64, data: impl AsRef<[u8]>) -> u64 {
    for &c in data.as_ref() {
        h = (h ^ c as u64).wrapping_mul(FNV64_PRIME);
    }
    h
}

pub fn fnv1a64_unwind(mut h: u64, suffix: impl AsRef<[u8]>) -> u64 {
    for &c in suffix.as_ref().iter().rev() {
        h = h.wrapping_mul(FNV64_PRIME_INV) ^ c as u64;
    }
    h
}

pub fn fnv1a32(data: impl AsRef<[u8]>) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &c in data.as_ref() {
        h = (h ^ c as u32).wrapping_mul(0x0100_0193);
    }
    h
}

pub fn djb2(data: impl AsRef<[u8]>) -> u32 {
    let mut h: u32 = 5381;
    for &c in data.as_ref() {
        h = h.wrapping_mul(33).wrapping_add(c as u32);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_inverse() {
        assert_eq!(FNV64_PRIME.wrapping_mul(FNV64_PRIME_INV), 1);
    }

    #[test]
    fn unwind_reverses_suffix() {
        let full = fnv1a64("tracks.Track_3.numberOfCorners");
        assert_eq!(
            fnv1a64_unwind(full, ".numberOfCorners"),
            fnv1a64("tracks.Track_3")
        );
    }

    #[test]
    fn known_vectors() {
        assert_eq!(fnv1a64(""), FNV64_BASIS);
        assert_eq!(fnv1a64("a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a32("a"), 0xe40c_292c);
        assert_eq!(djb2(""), 5381);
        assert_eq!(djb2("a"), 5381 * 33 + 97);
    }
}
