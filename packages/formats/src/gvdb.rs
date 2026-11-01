use crate::bytes::Reader;
use crate::hash::fnv1a64;
use crate::{Error, Result};

pub const MASK: &[u8; 30] = b"tG5MYmBZEZZnrH7yg2qTC4qj42Jcq9";
const MASK_START: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Float,
    String,
    Enum,
    Id,
    Bool,
    Time,
    Curve,
    List,
    Vector,
    Other(u8),
}

impl ValueType {
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            0 => Self::Int,
            1 => Self::Float,
            2 => Self::String,
            3 => Self::Enum,
            4 => Self::Id,
            5 => Self::Bool,
            6 => Self::Time,
            9 => Self::Curve,
            13 => Self::List,
            14 => Self::Vector,
            t => Self::Other(t),
        }
    }

    pub fn tag(self) -> u8 {
        match self {
            Self::Int => 0,
            Self::Float => 1,
            Self::String => 2,
            Self::Enum => 3,
            Self::Id => 4,
            Self::Bool => 5,
            Self::Time => 6,
            Self::Curve => 9,
            Self::List => 13,
            Self::Vector => 14,
            Self::Other(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub key: u64,
    pub tag: u8,
    pub value: Vec<u8>,
}

impl Record {
    pub fn new(path: &str, tag: u8, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: fnv1a64(path),
            tag,
            value: value.into(),
        }
    }

    pub fn key_hi(&self) -> u32 {
        (self.key >> 32) as u32
    }

    pub fn key_lo(&self) -> u32 {
        self.key as u32
    }

    pub fn value_type(&self) -> ValueType {
        ValueType::from_tag(self.tag)
    }

    pub fn text(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.value)
    }
}

fn transform<const ENCODE: bool>(bytes: &mut [u8], stream: usize) {
    let mut offset = (MASK_START + stream) % MASK.len();
    let mut rest = bytes;
    while !rest.is_empty() {
        let n = rest.len().min(MASK.len() - offset);
        let (chunk, tail) = rest.split_at_mut(n);
        for (b, &mask) in chunk.iter_mut().zip(&MASK[offset..offset + n]) {
            *b = if ENCODE {
                b.wrapping_add(mask)
            } else {
                b.wrapping_sub(mask)
            };
        }
        rest = tail;
        offset = 0;
    }
}

pub fn unmask(enc: &[u8], stream: usize) -> Vec<u8> {
    let mut out = enc.to_vec();
    transform::<false>(&mut out, stream);
    out
}

pub fn mask(plain: &[u8], stream: usize) -> Vec<u8> {
    let mut out = plain.to_vec();
    transform::<true>(&mut out, stream);
    out
}

pub fn instance_count_path(container: &str, row: &str) -> String {
    format!("{container}._{row}_instancecount_")
}

#[derive(Debug, Clone, Default)]
pub struct Gvdb {
    pub records: Vec<Record>,
}

impl Gvdb {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data, 0);
        let count = r.u32()? as usize;
        // A record takes at least 14 bytes, so reserve no more than the data can hold:
        // a foreign file's count is garbage and would otherwise reserve gigabytes.
        let mut records = Vec::with_capacity(count.min(data.len() / 14));
        let mut stream = 0;
        for _ in 0..count {
            let lo = r.u32()?;
            let hi = r.u32()?;
            let tag = r.u8()?;
            let n = r.u32()? as usize;
            let enc = r.take(n)?;
            if r.u8()? != 0 {
                return Err(Error::Format(format!("missing terminator at {:#x}", r.pos - 1)));
            }
            records.push(Record {
                key: (hi as u64) << 32 | lo as u64,
                tag,
                value: unmask(enc, stream),
            });
            stream += n;
        }
        if r.pos != data.len() {
            return Err(Error::Format(format!(
                "trailing bytes: parsed {} of {}",
                r.pos,
                data.len()
            )));
        }
        Ok(Self { records })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut order: Vec<&Record> = self.records.iter().collect();
        order.sort_by_key(|r| r.key_hi());
        write_records(order.iter().copied())
    }

    pub fn to_bytes_unsorted(&self) -> Vec<u8> {
        write_records(self.records.iter())
    }

    pub fn position(&self, key: u64) -> Option<usize> {
        self.records.iter().position(|r| r.key == key)
    }

    pub fn get_key(&self, key: u64) -> Option<&Record> {
        self.records.iter().find(|r| r.key == key)
    }

    pub fn get(&self, path: &str) -> Option<&Record> {
        self.get_key(fnv1a64(path))
    }

    pub fn get_text(&self, path: &str) -> Option<String> {
        self.get(path).map(|r| r.text().into_owned())
    }

    pub fn set(&mut self, path: &str, value: impl Into<Vec<u8>>, tag: Option<u8>) -> Result<bool> {
        let key = fnv1a64(path);
        let value = value.into();
        if let Some(r) = self.records.iter_mut().find(|r| r.key == key) {
            r.value = value;
            if let Some(t) = tag {
                r.tag = t;
            }
            return Ok(true);
        }
        let tag = tag.ok_or_else(|| Error::Format(format!("new key {path} needs a type tag")))?;
        self.records.push(Record { key, tag, value });
        Ok(false)
    }

    pub fn remove(&mut self, path: &str) -> Option<Record> {
        let i = self.position(fnv1a64(path))?;
        Some(self.records.remove(i))
    }

    pub fn instance_count(&self, container: &str, row: &str) -> Option<usize> {
        self.get_text(&instance_count_path(container, row))?
            .trim()
            .parse()
            .ok()
    }
}

fn write_records<'a>(records: impl ExactSizeIterator<Item = &'a Record> + Clone) -> Vec<u8> {
    let size = 4 + records.clone().map(|r| 14 + r.value.len()).sum::<usize>();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    let mut stream = 0;
    for r in records {
        out.extend_from_slice(&r.key_lo().to_le_bytes());
        out.extend_from_slice(&r.key_hi().to_le_bytes());
        out.push(r.tag);
        out.extend_from_slice(&(r.value.len() as u32).to_le_bytes());
        let start = out.len();
        out.extend_from_slice(&r.value);
        transform::<true>(&mut out[start..], stream);
        out.push(0);
        stream += r.value.len();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Gvdb {
        Gvdb {
            records: vec![
                Record::new("tracks.Track_3.instanceName", 4, "bahrain"),
                Record::new("tracks.Track_3.numberOfCorners", 0, "15"),
                Record::new("tracks.Track_3.allow_night_start", 5, "True"),
                Record::new("tracks._Track_instancecount_", 0, "43"),
            ],
        }
    }

    #[test]
    fn mask_round_trip() {
        let plain = b"Launched with Swofty Launcher";
        for pos in [0, 1, 29, 30, 12345] {
            assert_eq!(unmask(&mask(plain, pos), pos), plain);
        }
    }

    #[test]
    fn write_then_parse() {
        let db = sample();
        let bytes = db.to_bytes();
        let back = Gvdb::parse(&bytes).unwrap();
        assert_eq!(back.to_bytes(), bytes);
        assert_eq!(
            back.get_text("tracks.Track_3.instanceName").as_deref(),
            Some("bahrain")
        );
        assert_eq!(back.instance_count("tracks", "Track"), Some(43));
    }

    #[test]
    fn sorts_by_key_hi_stably() {
        let bytes = sample().to_bytes();
        let back = Gvdb::parse(&bytes).unwrap();
        let his: Vec<u32> = back.records.iter().map(Record::key_hi).collect();
        assert!(his.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn set_existing_and_new() {
        let mut db = sample();
        assert!(db.set("tracks.Track_3.numberOfCorners", "16", None).unwrap());
        assert!(db.set("tracks.Track_3.new_field", "x", None).is_err());
        assert!(!db.set("tracks.Track_3.new_field", "x", Some(2)).unwrap());
        assert_eq!(
            db.get_text("tracks.Track_3.numberOfCorners").as_deref(),
            Some("16")
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = sample().to_bytes();
        bytes.push(0);
        assert!(Gvdb::parse(&bytes).is_err());
    }

    #[test]
    fn rejects_a_count_the_data_cannot_hold_without_reserving_it() {
        let mut bytes = u32::MAX.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0; 64]);
        assert!(Gvdb::parse(&bytes).is_err());
    }
}
