//! Cursor sobre bytes que nunca causa pânico: todo acesso fora dos limites vira erro.

use super::AdfError;

/// Leitor sequencial little-endian sobre um slice.
pub(super) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Cria um leitor posicionado em `pos`.
    pub(super) fn at(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    /// Consome `len` bytes.
    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], AdfError> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|&end| end <= self.data.len())
            .ok_or(AdfError::OutOfBounds {
                offset: self.pos,
                wanted: len,
            })?;
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    /// Pula `len` bytes.
    pub(super) fn skip(&mut self, len: usize) -> Result<(), AdfError> {
        self.take(len).map(|_| ())
    }

    pub(super) fn u32(&mut self) -> Result<u32, AdfError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub(super) fn u64(&mut self) -> Result<u64, AdfError> {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(self.take(8)?);
        Ok(u64::from_le_bytes(buf))
    }

    /// Lê bytes até o próximo NUL (consumindo o NUL, sem incluí-lo no resultado).
    pub(super) fn cstr(&mut self) -> Result<&'a [u8], AdfError> {
        let rest = self.data.get(self.pos..).unwrap_or_default();
        let len = rest
            .iter()
            .position(|&b| b == 0)
            .ok_or(AdfError::UnterminatedString { offset: self.pos })?;
        let bytes = self.take(len)?;
        self.skip(1)?;
        Ok(bytes)
    }
}

/// Converte bytes Latin-1 em `String` (cada byte vira o ponto de código igual).
pub(super) fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_little_endian_integers() {
        let data = [1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        let mut r = Reader::at(&data, 0);
        assert_eq!(r.u32().unwrap(), 1);
        assert_eq!(r.u64().unwrap(), 2);
    }

    #[test]
    fn out_of_bounds_is_an_error_not_a_panic() {
        let mut r = Reader::at(&[1, 2, 3], 0);
        assert!(matches!(
            r.u32(),
            Err(AdfError::OutOfBounds {
                offset: 0,
                wanted: 4
            })
        ));
    }

    #[test]
    fn start_beyond_end_is_an_error() {
        assert!(Reader::at(&[1], 99).take(1).is_err());
        assert!(Reader::at(&[1], 99).cstr().is_err());
    }

    #[test]
    fn cstr_stops_at_nul_and_consumes_it() {
        let mut r = Reader::at(b"abc\0def\0", 0);
        assert_eq!(r.cstr().unwrap(), b"abc");
        assert_eq!(r.cstr().unwrap(), b"def");
    }

    #[test]
    fn cstr_without_terminator_fails() {
        assert!(matches!(
            Reader::at(b"abc", 0).cstr(),
            Err(AdfError::UnterminatedString { offset: 0 })
        ));
    }

    #[test]
    fn latin1_maps_bytes_to_chars() {
        assert_eq!(latin1(&[0x61, 0xe9]), "aé");
    }
}
