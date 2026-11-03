//! Leituras little-endian com checagem de limites, sem pânico.

/// `len` bytes a partir de `offset`, ou `None` se passar do fim.
pub(crate) fn slice(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    data.get(offset..offset.checked_add(len)?)
}

pub(crate) fn u16_at(data: &[u8], offset: usize) -> Option<u16> {
    let b = slice(data, offset, 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

pub(crate) fn u32_at(data: &[u8], offset: usize) -> Option<u32> {
    let b = slice(data, offset, 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

pub(crate) fn u64_at(data: &[u8], offset: usize) -> Option<u64> {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(slice(data, offset, 8)?);
    Some(u64::from_le_bytes(buf))
}

/// Texto terminado em NUL que começa em `offset` (sem o NUL).
pub(crate) fn cstr_at(data: &[u8], offset: usize) -> Option<&[u8]> {
    let rest = data.get(offset..)?;
    let len = rest.iter().position(|&b| b == 0)?;
    Some(&rest[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_within_bounds() {
        let data = [1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(u16_at(&data, 0), Some(1));
        assert_eq!(u32_at(&data, 2), Some(2));
        assert_eq!(u64_at(&data, 6), Some(3));
        assert_eq!(slice(&data, 12, 2), Some(&[0, 0][..]));
    }

    #[test]
    fn reads_past_the_end_return_none() {
        let data = [1, 2, 3];
        assert_eq!(u16_at(&data, 2), None);
        assert_eq!(u32_at(&data, 0), None);
        assert_eq!(u64_at(&data, 0), None);
        assert_eq!(slice(&data, 99, 1), None);
        assert_eq!(slice(&data, 1, usize::MAX), None);
    }

    #[test]
    fn cstr_stops_at_nul() {
        assert_eq!(cstr_at(b"ab\0cd\0", 3), Some(&b"cd"[..]));
        assert_eq!(cstr_at(b"abc", 0), None);
        assert_eq!(cstr_at(b"abc", 9), None);
    }
}
