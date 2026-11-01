use std::io::{Read, Seek, SeekFrom};

use aes::Aes256;
use aes::cipher::{BlockCipherDecrypt, KeyInit};

use crate::bytes::Reader;
use crate::{Error, Result};

pub const MAGIC: &[u8; 4] = b"NeFS";
pub const VERSION_160: u32 = 0x10600;

const INTRO: usize = 128;
const TRANSFORM_NONE: u16 = 0;
const TRANSFORM_AES: u16 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub start: u64,
    pub size: u32,
    first_block: u32,
}

#[derive(Debug, Clone, Copy)]
struct Block {
    end: u32,
    transform: u16,
}

pub struct Archive {
    pub volume: String,
    pub block_size: u32,
    pub entries: Vec<Entry>,
    blocks: Vec<Block>,
    cipher: Aes256,
}

impl std::fmt::Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archive")
            .field("volume", &self.volume)
            .field("block_size", &self.block_size)
            .field("entries", &self.entries.len())
            .finish()
    }
}

fn u32_at(data: &[u8], pos: usize) -> Result<u32> {
    Reader::new(data, pos).u32()
}

fn table<T>(
    data: &[u8],
    start: u32,
    end: u32,
    size: usize,
    row: impl Fn(&mut Reader) -> Result<T>,
) -> Result<Vec<T>> {
    let (start, end) = (start as usize, end as usize);
    if end < start || end > data.len() {
        return Err(Error::Format(format!(
            "NeFS table {start:#x}..{end:#x} is outside the header"
        )));
    }
    let mut r = Reader::new(data, start);
    (0..(end - start) / size).map(|_| row(&mut r)).collect()
}

fn hex_key(text: &[u8]) -> Result<[u8; 32]> {
    let digit = |c: u8| {
        (c as char)
            .to_digit(16)
            .map(|d| d as u8)
            .ok_or_else(|| Error::Format("NeFS key is not hex".into()))
    };
    let mut key = [0; 32];
    for (i, pair) in text.chunks_exact(2).enumerate() {
        key[i] = digit(pair[0])? << 4 | digit(pair[1])?;
    }
    Ok(key)
}

impl Archive {
    pub fn find(exe: &mut (impl Read + Seek)) -> Result<Vec<u64>> {
        const CHUNK: usize = 8 << 20;
        let mut found = Vec::new();
        let mut buf = vec![0; CHUNK + INTRO];
        let mut base = 0u64;
        let mut keep = 0;
        exe.seek(SeekFrom::Start(0))?;
        loop {
            let mut len = keep;
            while len < buf.len() {
                match exe.read(&mut buf[len..])? {
                    0 => break,
                    n => len += n,
                }
            }
            if len < 8 {
                return Ok(found);
            }
            let window = &buf[..len];
            let mut pos = 0;
            while pos + 108 <= len {
                if &window[pos..pos + 4] == MAGIC && window[pos + 104..pos + 108] == VERSION_160.to_le_bytes()
                {
                    found.push(base + pos as u64);
                }
                pos += 4;
            }
            if len < buf.len() {
                return Ok(found);
            }
            buf.copy_within(pos..len, 0);
            keep = len - pos;
            base += pos as u64;
        }
    }

    pub fn read_header(exe: &mut (impl Read + Seek), offset: u64) -> Result<Self> {
        let mut intro = [0; INTRO];
        exe.seek(SeekFrom::Start(offset))?;
        exe.read_exact(&mut intro)?;
        let size = u32_at(&intro, 100)? as usize;
        if !(INTRO + 40..=64 << 20).contains(&size) {
            return Err(Error::Format(format!("NeFS header size {size} is out of range")));
        }
        let mut header = vec![0; size];
        exe.seek(SeekFrom::Start(offset))?;
        exe.read_exact(&mut header)?;
        Self::parse(&header)
    }

    pub fn open_exe(exe: &mut (impl Read + Seek)) -> Result<Self> {
        let mut last = Error::Format("no NeFS 1.6 header in the exe".into());
        for offset in Self::find(exe)? {
            match Self::read_header(exe, offset) {
                Ok(a) if !a.entries.is_empty() => return Ok(a),
                Ok(_) => {}
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn parse(header: &[u8]) -> Result<Self> {
        if !header.starts_with(MAGIC) {
            return Err(Error::Magic {
                offset: 0,
                expected: "NeFS",
            });
        }
        let version = u32_at(header, 104)?;
        if version != VERSION_160 {
            return Err(Error::Format(format!("NeFS version {version:#x} is not 1.6")));
        }
        let key = hex_key(header.get(36..100).ok_or(Error::Eof(36))?)?;
        let mut toc = Reader::new(header, INTRO);
        let volumes = toc.u16()?;
        let _hash_block = toc.u16()?;
        let block_size = (toc.u16()? as u32) << 15;
        let _split = toc.u16()?;
        let mut at = [0u32; 8];
        for a in &mut at {
            *a = toc.u32()?;
        }
        let [entry_at, _, shared_at, _, name_at, block_at, volume_at, _] = at;
        if volumes == 0 || block_size == 0 {
            return Err(Error::Format("NeFS header has no volume or block size".into()));
        }

        let raw = table(header, entry_at, shared_at, 20, |r| {
            let lo = r.u32()? as u64;
            let hi = r.u32()? as u64;
            let shared = r.u32()?;
            let first_block = r.u32()?;
            r.u32()?;
            Ok((hi << 32 | lo, shared, first_block))
        })?;
        let shared = table(header, shared_at, name_at, 20, |r| {
            let parent = r.u32()?;
            r.u32()?;
            let name = r.u32()?;
            let size = r.u32()?;
            r.u32()?;
            Ok((parent, name, size))
        })?;
        let names = header
            .get(name_at as usize..block_at as usize)
            .ok_or_else(|| Error::Format("NeFS name table is outside the header".into()))?;
        let blocks = table(header, block_at, volume_at, 8, |r| {
            let end = r.u32()?;
            let transform = r.u16()?;
            r.u16()?;
            Ok(Block { end, transform })
        })?;
        let mut v = Reader::new(header, volume_at as usize);
        let _volume_size = v.u64()?;
        let volume_name = v.u32()?;

        let name = |off: u32| -> Result<String> {
            let rest = names
                .get(off as usize..)
                .ok_or_else(|| Error::Format(format!("NeFS name offset {off:#x} is outside the table")))?;
            let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
            Ok(String::from_utf8_lossy(&rest[..end]).into_owned())
        };
        let info = |i: usize| -> Result<(u64, u32, u32, u32, u32)> {
            let (start, s, first_block) = raw[i];
            let &(parent, name, size) = shared
                .get(s as usize)
                .ok_or_else(|| Error::Format(format!("NeFS entry {i} names shared info {s}")))?;
            Ok((start, first_block, parent, name, size))
        };
        let mut entries = Vec::with_capacity(raw.len());
        for i in 0..raw.len() {
            let (start, first_block, _, _, size) = info(i)?;
            let mut parts = Vec::new();
            let mut at = i;
            loop {
                let (_, _, parent, n, _) = info(at)?;
                parts.push(name(n)?);
                if parent as usize == at || parts.len() > raw.len() {
                    break;
                }
                if parent as usize >= raw.len() {
                    return Err(Error::Format(format!("NeFS entry {at} has parent {parent}")));
                }
                at = parent as usize;
            }
            parts.reverse();
            entries.push(Entry {
                path: parts.join("/"),
                start,
                size,
                first_block,
            });
        }
        Ok(Self {
            volume: name(volume_name)?,
            block_size,
            entries,
            blocks,
            cipher: Aes256::new(&key.into()),
        })
    }

    pub fn entry(&self, path: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.path.eq_ignore_ascii_case(path))
    }

    pub fn read(&self, entry: &Entry, data: &mut (impl Read + Seek)) -> Result<Vec<u8>> {
        let count = entry.size.div_ceil(self.block_size) as usize;
        let first = entry.first_block as usize;
        let blocks = self
            .blocks
            .get(first..first + count)
            .ok_or_else(|| Error::Format(format!("{} lists blocks past the block table", entry.path)))?;
        let mut out = Vec::with_capacity(entry.size as usize);
        let mut prev = 0u32;
        data.seek(SeekFrom::Start(entry.start))?;
        for b in blocks {
            let len = b
                .end
                .checked_sub(prev)
                .ok_or_else(|| Error::Format(format!("{} has blocks out of order", entry.path)))?;
            prev = b.end;
            let mut chunk = vec![0; len as usize];
            data.read_exact(&mut chunk)?;
            match b.transform {
                TRANSFORM_NONE => {}
                TRANSFORM_AES => {
                    if !chunk.len().is_multiple_of(16) {
                        return Err(Error::Format(format!(
                            "{} has an AES block of {} bytes",
                            entry.path,
                            chunk.len()
                        )));
                    }
                    for block in chunk.chunks_exact_mut(16) {
                        self.cipher.decrypt_block(block.try_into().expect("16 bytes"));
                    }
                }
                t => {
                    return Err(Error::Format(format!(
                        "{} uses block transform {t}, which is not supported",
                        entry.path
                    )));
                }
            }
            let want = (entry.size as usize - out.len()).min(self.block_size as usize);
            out.extend_from_slice(&chunk[..want.min(chunk.len())]);
        }
        if out.len() != entry.size as usize {
            return Err(Error::Format(format!(
                "{} read {} of {} bytes",
                entry.path,
                out.len(),
                entry.size
            )));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::BlockCipherEncrypt;
    use std::io::Cursor;

    const KEY: &[u8; 64] = b"94C813A02BD715CB4B9189D772B2E9D2BC0158EDC8831C55CEDA3C868283C462";

    fn put(out: &mut Vec<u8>, vals: &[u32]) {
        for v in vals {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }

    fn sample(content: &[u8], block_size: u32) -> (Vec<u8>, Vec<u8>) {
        let cipher = Aes256::new(&hex_key(KEY).unwrap().into());
        let mut data = vec![0xee; 32];
        let mut ends = Vec::new();
        for chunk in content.chunks(block_size as usize) {
            let mut c = chunk.to_vec();
            c.resize(chunk.len().div_ceil(16) * 16, 0);
            for b in c.chunks_exact_mut(16) {
                cipher.encrypt_block(b.try_into().unwrap());
            }
            data.extend_from_slice(&c);
            ends.push((data.len() - 32) as u32);
        }

        let names = b"game.dat\0localisation\0language_eng.lng\0";
        let entry_at = (INTRO + 0x80) as u32;
        let shared_at = entry_at + 2 * 20;
        let name_at = shared_at + 2 * 20;
        let block_at = name_at + names.len() as u32;
        let volume_at = block_at + 8 * ends.len() as u32;
        let size = volume_at + 16;

        let mut h = Vec::new();
        h.extend_from_slice(MAGIC);
        h.resize(36, 0);
        h.extend_from_slice(KEY);
        put(&mut h, &[size, VERSION_160, 2, 0]);
        h.resize(INTRO, 0);
        h.extend_from_slice(&1u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&((block_size >> 15) as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        put(
            &mut h,
            &[entry_at, 0, shared_at, 0, name_at, block_at, volume_at, 0],
        );
        h.resize(entry_at as usize, 0);
        put(&mut h, &[0, 0, 0, 0, 0]);
        put(&mut h, &[32, 0, 1, 0, 1]);
        put(&mut h, &[0, 1, 9, 0, 0]);
        put(&mut h, &[0, 1, 22, content.len() as u32, 1]);
        h.extend_from_slice(names);
        for e in ends {
            h.extend_from_slice(&e.to_le_bytes());
            h.extend_from_slice(&TRANSFORM_AES.to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes());
        }
        h.extend_from_slice(&(data.len() as u64).to_le_bytes());
        put(&mut h, &[0, 0]);
        (h, data)
    }

    #[test]
    fn reads_an_encrypted_entry() {
        let content: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
        let (header, data) = sample(&content, 1 << 15);
        let mut exe = vec![0x90; 4096];
        exe.extend_from_slice(&header);
        exe.extend_from_slice(&[0; 300]);
        let mut exe = Cursor::new(exe);
        assert_eq!(Archive::find(&mut exe).unwrap(), vec![4096]);
        let archive = Archive::open_exe(&mut exe).unwrap();
        assert_eq!(archive.volume, "game.dat");
        assert_eq!(archive.block_size, 1 << 15);
        let paths: Vec<&str> = archive.entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["localisation", "localisation/language_eng.lng"]);
        let entry = archive.entry("localisation/language_eng.lng").unwrap();
        assert_eq!(archive.read(entry, &mut Cursor::new(data)).unwrap(), content);
    }

    #[test]
    fn rejects_other_versions_and_transforms() {
        let (mut header, data) = sample(b"LNGT", 1 << 15);
        let block = header.len() - 16 - 8 + 4;
        header[block..block + 2].copy_from_slice(&7u16.to_le_bytes());
        let archive = Archive::parse(&header).unwrap();
        assert!(archive.read(&archive.entries[1], &mut Cursor::new(data)).is_err());
        header[104..108].copy_from_slice(&0x20000u32.to_le_bytes());
        assert!(Archive::parse(&header).is_err());
    }
}
