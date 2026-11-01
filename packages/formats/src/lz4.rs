use crate::{Error, Result};

fn length(src: &[u8], at: &mut usize, mut n: usize) -> Result<usize> {
    if n == 15 {
        loop {
            let b = *src.get(*at).ok_or(Error::Eof(*at))?;
            *at += 1;
            n += b as usize;
            if b != 255 {
                break;
            }
        }
    }
    Ok(n)
}

pub fn decompress_block(src: &[u8], size: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(size);
    let mut at = 0;
    while at < src.len() {
        let token = src[at];
        at += 1;
        let literals = length(src, &mut at, (token >> 4) as usize)?;
        let run = src.get(at..at + literals).ok_or(Error::Eof(src.len()))?;
        out.extend_from_slice(run);
        at += literals;
        if at == src.len() {
            break;
        }
        let offset = src
            .get(at..at + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]) as usize)
            .ok_or(Error::Eof(src.len()))?;
        at += 2;
        let len = length(src, &mut at, (token & 15) as usize)? + 4;
        if offset == 0 || offset > out.len() {
            return Err(Error::Format(format!(
                "lz4 match offset {offset} is out of range"
            )));
        }
        if out.len() + len > size {
            return Err(Error::Format(format!("lz4 block is longer than {size} bytes")));
        }
        if offset == 1 {
            out.resize(out.len() + len, out[out.len() - 1]);
        } else {
            let start = out.len() - offset;
            let mut remaining = len;
            while remaining != 0 {
                let count = remaining.min(out.len() - start);
                out.extend_from_within(start..start + count);
                remaining -= count;
            }
        }
    }
    if out.len() != size {
        return Err(Error::Format(format!(
            "lz4 block gave {} bytes, expected {size}",
            out.len()
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_and_overlapping_matches() {
        let src = [0x32, b'a', b'b', b'c', 3, 0, 0x50, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(decompress_block(&src, 14).unwrap(), b"abcabcabchello");
    }

    #[test]
    fn long_lengths() {
        let mut src = vec![0x1f, b'x', 1, 0, 255, 10];
        src.extend([0x10, b'!']);
        let out = decompress_block(&src, 1 + 15 + 255 + 10 + 4 + 1).unwrap();
        assert!(out[..out.len() - 1].iter().all(|&b| b == b'x'));
        assert_eq!(out.last(), Some(&b'!'));
    }

    #[test]
    fn rejects_bad_offsets_and_sizes() {
        assert!(decompress_block(&[0x10, b'a', 5, 0, 0x10, b'b'], 7).is_err());
        assert!(decompress_block(&[0x30, b'a', b'b', b'c'], 4).is_err());
        assert!(decompress_block(&[0x30, b'a'], 3).is_err());
    }
}
