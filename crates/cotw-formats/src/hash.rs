//! `hashlittle` (Bob Jenkins, lookup3), o hash que o jogo usa para nomes de arquivo.
//!
//! Porta fiel de `research/reference/hash.py`.

/// Calcula o `hashlittle` de `data` com a semente `initval`.
///
/// O jogo usa `initval = 0` para caminhos de arquivo.
#[must_use]
pub fn hashlittle(data: &[u8], initval: u32) -> u32 {
    // O algoritmo original usa o tamanho módulo 2^32, então truncar é intencional.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "lookup3 usa o tamanho módulo 2^32"
    )]
    let len = data.len() as u32;
    let seed = 0xdead_beef_u32.wrapping_add(len).wrapping_add(initval);
    let (mut a, mut b, mut c) = (seed, seed, seed);

    // Blocos de 12 bytes só entram na mistura enquanto sobrar MAIS de 12 bytes;
    // os últimos 1 a 12 bytes vão para a etapa final.
    let mut rest = data;
    while rest.len() > 12 {
        let (wa, wb, wc) = words(&rest[..12]);
        a = a.wrapping_add(wa);
        b = b.wrapping_add(wb);
        c = c.wrapping_add(wc);
        mix(&mut a, &mut b, &mut c);
        rest = &rest[12..];
    }

    if rest.is_empty() {
        return c;
    }

    let (wa, wb, wc) = words(rest);
    a = a.wrapping_add(wa);
    b = b.wrapping_add(wb);
    c = c.wrapping_add(wc);
    finalize(a, b, c)
}

/// Lê até 12 bytes como três palavras little-endian, completando com zeros.
fn words(bytes: &[u8]) -> (u32, u32, u32) {
    let mut block = [0u8; 12];
    block[..bytes.len()].copy_from_slice(bytes);
    let word = |i: usize| u32::from_le_bytes([block[i], block[i + 1], block[i + 2], block[i + 3]]);
    (word(0), word(4), word(8))
}

fn mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*c);
    *a ^= c.rotate_left(4);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= a.rotate_left(6);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= b.rotate_left(8);
    *b = b.wrapping_add(*a);
    *a = a.wrapping_sub(*c);
    *a ^= c.rotate_left(16);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= a.rotate_left(19);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= b.rotate_left(4);
    *b = b.wrapping_add(*a);
}

fn finalize(mut a: u32, mut b: u32, mut c: u32) -> u32 {
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(14));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(11));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(25));
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(16));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(4));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(14));
    c ^= b;
    c.wrapping_sub(b.rotate_left(24))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_seed() {
        assert_eq!(hashlittle(b"", 0), 0xdead_beef);
    }

    #[test]
    fn matches_jenkins_reference_vectors() {
        let text = b"Four score and seven years ago";
        assert_eq!(hashlittle(text, 0), 0x1777_0551);
        assert_eq!(hashlittle(text, 1), 0xcd62_8161);
    }

    #[test]
    fn handles_block_boundaries() {
        // 12 bytes vai direto para a etapa final; 13 passa antes pela mistura.
        assert_eq!(hashlittle(&[b'a'; 12], 0), 0x6bcd_f787);
        assert_eq!(hashlittle(&[b'a'; 13], 0), 0x8209_31ff);
        assert_eq!(hashlittle(b"abc", 0), 0x0e39_7631);
    }

    #[test]
    fn different_seeds_give_different_hashes() {
        assert_ne!(hashlittle(b"abc", 0), hashlittle(b"abc", 7));
    }
}
