use std::collections::HashSet;

use crate::{Error, Result};

pub const MAGIC: &[u8; 5] = b"\x01BXML";
pub const STATS_MAGIC: &[u8; 4] = b"TSMD";
pub const STATS_LEN: usize = 120;

// The game sizes its string pools for a bxml fragment from the TSMD stats beside it,
// so data with more strings than its stats count overruns them.
pub fn stale_stats(data: &[u8], stats: &[u8]) -> bool {
    if !data.starts_with(MAGIC) || stats.len() != STATS_LEN || !stats.starts_with(STATS_MAGIC) {
        return false;
    }
    let version = u32::from_le_bytes(stats[4..8].try_into().expect("4 bytes"));
    Document::parse(data).map_or(true, |d| d.string_stats(version) != stats)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Element,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub kind: Kind,
    pub pad: u8,
    pub name: Vec<u8>,
    pub attrs: Vec<(Vec<u8>, Vec<u8>)>,
    pub children: Vec<Element>,
    pub end_pad: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub root: Element,
    pub tail: Vec<u8>,
}

impl Element {
    pub fn attr(&self, key: &[u8]) -> Option<&[u8]> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }

    pub fn set_attr(&mut self, key: &[u8], value: &[u8]) {
        match self.attrs.iter_mut().find(|(k, _)| k == key) {
            Some((_, v)) => *v = value.to_vec(),
            None => self.attrs.push((key.to_vec(), value.to_vec())),
        }
    }

    pub fn children_named<'a, 'n: 'a>(&'a self, name: &'n [u8]) -> impl Iterator<Item = &'a Element> + 'a {
        self.children
            .iter()
            .filter(move |c| c.kind == Kind::Element && c.name == name)
    }

    pub fn child(&self, name: &[u8]) -> Option<&Element> {
        self.children
            .iter()
            .find(|c| c.kind == Kind::Element && c.name == name)
    }

    pub fn child_mut(&mut self, name: &[u8]) -> Option<&mut Element> {
        self.children
            .iter_mut()
            .find(|c| c.kind == Kind::Element && c.name == name)
    }

    fn walk<'a>(&'a self, out: &mut Vec<&'a Element>) {
        out.push(self);
        for c in &self.children {
            c.walk(out);
        }
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn header(&mut self) -> Result<(u16, u8, u8, u16)> {
        let b = self
            .data
            .get(self.pos..self.pos + 6)
            .ok_or(Error::Eof(self.pos))?;
        self.pos += 6;
        Ok((
            u16::from_le_bytes([b[0], b[1]]),
            b[2],
            b[3],
            u16::from_le_bytes([b[4], b[5]]),
        ))
    }

    fn cstr(&mut self) -> Result<Vec<u8>> {
        let rest = self.data.get(self.pos..).ok_or(Error::Eof(self.pos))?;
        let end = rest.iter().position(|&b| b == 0).ok_or(Error::Eof(self.pos))?;
        let s = rest[..end].to_vec();
        self.pos += end + 1;
        Ok(s)
    }

    fn node(&mut self) -> Result<Option<Element>> {
        let start = self.pos;
        let (len, kind, pad, nattr) = self.header()?;
        let check = |c: &Cursor| -> Result<()> {
            if c.pos - start - 2 != len as usize {
                return Err(Error::Format(format!(
                    "bxml record at {start:#x} has a wrong length"
                )));
            }
            Ok(())
        };
        match kind {
            0 => {
                let name = self.cstr()?;
                let mut attrs = Vec::with_capacity(nattr as usize);
                for _ in 0..nattr {
                    attrs.push((self.cstr()?, self.cstr()?));
                }
                check(self)?;
                let mut children = Vec::new();
                let end_pad = loop {
                    let at = self.pos;
                    match self.node()? {
                        Some(c) => children.push(c),
                        None => break self.data[at + 3],
                    }
                };
                Ok(Some(Element {
                    kind: Kind::Element,
                    pad,
                    name,
                    attrs,
                    children,
                    end_pad,
                }))
            }
            1 => {
                let text = self.cstr()?;
                check(self)?;
                Ok(Some(Element {
                    kind: Kind::Text,
                    pad,
                    name: text,
                    attrs: Vec::new(),
                    children: Vec::new(),
                    end_pad: 0,
                }))
            }
            5 if len == 4 && nattr == 0 => Ok(None),
            _ => Err(Error::Format(format!("bxml record type {kind} at {start:#x}"))),
        }
    }
}

impl Document {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if !data.starts_with(MAGIC) {
            return Err(Error::Magic {
                offset: 0,
                expected: "\\x01BXML",
            });
        }
        let mut c = Cursor { data, pos: 5 };
        let root = c
            .node()?
            .ok_or_else(|| Error::Format("bxml has no root element".into()))?;
        let mut tail = Vec::new();
        while c.pos < data.len() {
            let at = c.pos;
            let (len, kind, pad, nattr) = c.header()?;
            if kind != 6 || len != 4 || nattr != 0 {
                return Err(Error::Format(format!("unexpected bxml record {kind} at {at:#x}")));
            }
            tail.push(pad);
        }
        Ok(Self { root, tail })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        fn rec(out: &mut Vec<u8>, kind: u8, pad: u8, body: &[u8], nattr: usize) {
            out.extend_from_slice(&((body.len() + 4) as u16).to_le_bytes());
            out.push(kind);
            out.push(pad);
            out.extend_from_slice(&(nattr as u16).to_le_bytes());
            out.extend_from_slice(body);
        }
        fn node(out: &mut Vec<u8>, e: &Element) {
            match e.kind {
                Kind::Element => {
                    let mut body = e.name.clone();
                    body.push(0);
                    for (k, v) in &e.attrs {
                        body.extend_from_slice(k);
                        body.push(0);
                        body.extend_from_slice(v);
                        body.push(0);
                    }
                    rec(out, 0, e.pad, &body, e.attrs.len());
                    for c in &e.children {
                        node(out, c);
                    }
                    rec(out, 5, e.end_pad, &[], 0);
                }
                Kind::Text => {
                    let mut body = e.name.clone();
                    body.push(0);
                    rec(out, 1, e.pad, &body, 0);
                }
            }
        }
        let mut out = MAGIC.to_vec();
        node(&mut out, &self.root);
        for &pad in &self.tail {
            rec(&mut out, 6, pad, &[], 0);
        }
        out
    }

    pub fn string_stats(&self, version: u32) -> Vec<u8> {
        let mut all = Vec::new();
        self.root.walk(&mut all);
        let els: Vec<&Element> = all.iter().copied().filter(|e| e.kind == Kind::Element).collect();
        let texts: Vec<&[u8]> = all
            .iter()
            .filter(|e| e.kind == Kind::Text)
            .map(|e| e.name.as_slice())
            .collect();
        let keys: Vec<&[u8]> = els
            .iter()
            .flat_map(|e| e.attrs.iter().map(|(k, _)| k.as_slice()))
            .collect();
        let vals: Vec<&[u8]> = els
            .iter()
            .flat_map(|e| e.attrs.iter().map(|(_, v)| v.as_slice()))
            .collect();
        let names: Vec<&[u8]> = els.iter().map(|e| e.name.as_slice()).collect();
        fn join<'s>(parts: &[&Vec<&'s [u8]>]) -> Vec<&'s [u8]> {
            parts.iter().flat_map(|p| p.iter().copied()).collect()
        }
        let cats: [Vec<&[u8]>; 7] = [
            keys.clone(),
            vals.clone(),
            names.clone(),
            texts.clone(),
            join(&[&names, &keys]),
            join(&[&vals, &texts]),
            join(&[&names, &keys, &vals, &texts]),
        ];
        let mut out = b"TSMD".to_vec();
        out.extend_from_slice(&version.to_le_bytes());
        let mut put = |n: usize, bytes: usize| {
            out.extend_from_slice(&(n as u32).to_le_bytes());
            out.extend_from_slice(&(bytes as u32).to_le_bytes());
        };
        for c in &cats {
            put(c.len(), c.iter().map(|s| s.len()).sum());
        }
        for c in &cats {
            let u: HashSet<&[u8]> = c.iter().copied().collect();
            put(u.len(), u.iter().map(|s| s.len()).sum());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let leaf = |name: &[u8], v: &[u8]| Element {
            kind: Kind::Element,
            pad: 0,
            name: name.to_vec(),
            attrs: vec![(b"value".to_vec(), v.to_vec())],
            children: Vec::new(),
            end_pad: 0,
        };
        let doc = Document {
            root: Element {
                kind: Kind::Element,
                pad: 0,
                name: b"VTF".to_vec(),
                attrs: Vec::new(),
                children: vec![leaf(b"Track", b"bahrain"), leaf(b"MediumDeployFactor", b"0.8")],
                end_pad: 0,
            },
            tail: vec![0, 0],
        };
        doc.to_bytes()
    }

    #[test]
    fn round_trip_and_stats() {
        let bytes = sample();
        let doc = Document::parse(&bytes).unwrap();
        assert_eq!(doc.to_bytes(), bytes);
        assert_eq!(
            doc.root.child(b"Track").unwrap().attr(b"value"),
            Some(&b"bahrain"[..])
        );
        let stats = doc.string_stats(1);
        assert_eq!(stats.len(), 4 + 4 + 28 * 4);
        assert_eq!(u32::from_le_bytes(stats[8..12].try_into().unwrap()), 2);
    }

    #[test]
    fn stats_go_stale_when_the_data_changes() {
        let bytes = sample();
        let stats = Document::parse(&bytes).unwrap().string_stats(1);
        assert_eq!(stats.len(), STATS_LEN);
        assert!(!stale_stats(&bytes, &stats));
        let mut doc = Document::parse(&bytes).unwrap();
        doc.root.children[0].set_attr(b"value", b"bahrain_longer_name");
        assert!(stale_stats(&doc.to_bytes(), &stats));
        assert!(stale_stats(&bytes[..bytes.len() / 2], &stats));
        assert!(!stale_stats(b"!pkg", &stats));
        assert!(!stale_stats(&bytes, &stats[..100]));
    }

    #[test]
    fn rejects_other_data() {
        assert!(Document::parse(b"!pkg").is_err());
    }
}
