use std::collections::HashMap;

use swofty_formats::gvdb::{Gvdb, Record};
use swofty_formats::hash::{fnv1a64, fnv1a64_from};

use crate::{Error, Result};

pub fn key(path: &str) -> u64 {
    fnv1a64(path)
}

pub fn at(h: u64, suffix: &str) -> u64 {
    fnv1a64_from(h, suffix)
}

pub fn field(row: u64, name: &str) -> u64 {
    at(row, &format!(".{name}"))
}

pub fn row(table: u64, i: usize) -> u64 {
    at(table, &i.to_string())
}

pub fn sub_table(row: u64, name: &str) -> u64 {
    at(row, &format!(".{name}_"))
}

pub fn sub_count(row: u64, name: &str) -> u64 {
    at(row, &format!("._{name}_instancecount_"))
}

pub struct Db<'a> {
    pub db: &'a mut Gvdb,
    pos: HashMap<u64, usize>,
    pub added: usize,
    pub changed: usize,
}

impl<'a> Db<'a> {
    pub fn new(db: &'a mut Gvdb) -> Self {
        let pos = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
        Self {
            db,
            pos,
            added: 0,
            changed: 0,
        }
    }

    pub fn get(&self, key: u64) -> Option<&Record> {
        self.pos.get(&key).map(|&i| &self.db.records[i])
    }

    pub fn has(&self, key: u64) -> bool {
        self.pos.contains_key(&key)
    }

    pub fn text(&self, key: u64) -> Option<String> {
        self.get(key).map(|r| r.text().into_owned())
    }

    pub fn int(&self, key: u64) -> Option<i64> {
        self.text(key)?.trim().parse().ok()
    }

    pub fn put(&mut self, key: u64, tag: u8, value: &[u8]) {
        match self.pos.get(&key) {
            Some(&i) => {
                let r = &mut self.db.records[i];
                if r.value != value || r.tag != tag {
                    r.value = value.to_vec();
                    r.tag = tag;
                    self.changed += 1;
                }
            }
            None => {
                self.pos.insert(key, self.db.records.len());
                self.db.records.push(Record {
                    key,
                    tag,
                    value: value.to_vec(),
                });
                self.added += 1;
            }
        }
    }

    pub fn set_text(&mut self, key: u64, value: &str, what: &str) -> Result<()> {
        let tag = self
            .get(key)
            .map(|r| r.tag)
            .ok_or_else(|| Error::Resolve(format!("{what}: no such key")))?;
        self.put(key, tag, value.as_bytes());
        Ok(())
    }

    pub fn copy(&mut self, from: u64, to: u64) -> bool {
        match self.get(from) {
            Some(r) => {
                let (tag, value) = (r.tag, r.value.clone());
                self.put(to, tag, &value);
                true
            }
            None => false,
        }
    }

    pub fn count(&self, key: u64) -> Option<usize> {
        self.int(key).and_then(|n| usize::try_from(n).ok())
    }

    pub fn rows(&self, table: u64, count: u64) -> Vec<u64> {
        (0..self.count(count).unwrap_or(0))
            .map(|i| row(table, i))
            .collect()
    }
}
