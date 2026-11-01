//! F1 25 `GfxMaterialRes` (`.material`) constants: read them and change their values in place.
//!
//! Layout: `re/db/subsystems/gfx_material.md`. A record is 9 u32s after a 0x20-byte header; a
//! constant's value sits in the value blob at `base + at`, where `base = 0x20 + 36 * nparams`.

use std::fmt;

const HEADER: usize = 0x20;
const RECORD: usize = 36;
const CONSTANT: u32 = 1;
const FLOAT: u32 = 3;
const UINT: u32 = 6;
const VALUE: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Truncated,
    NoConstant(String),
    NotAValue(String),
    Width { name: String, want: usize, got: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated => f.write_str("the material is truncated or not an F1 25 material"),
            Error::NoConstant(n) => write!(f, "the material has no constant {n}"),
            Error::NotAValue(n) => write!(f, "{n} is a bindless heap index, not a value"),
            Error::Width { name, want, got } => {
                write!(f, "{name} takes {want} value(s), not {got}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// One constant of a material.
#[derive(Debug, Clone, PartialEq)]
pub struct Constant {
    pub name: String,
    /// True for floats, false for uints.
    pub float: bool,
    pub width: usize,
    /// Where its value starts in the material's bytes.
    pub at: usize,
}

fn u32_at(b: &[u8], off: usize) -> Result<u32, Error> {
    b.get(off..off + 4)
        .map(|s| u32::from_le_bytes(s.try_into().expect("4 bytes")))
        .ok_or(Error::Truncated)
}

fn cstr(b: &[u8], off: usize) -> Result<String, Error> {
    let rest = b.get(off..).ok_or(Error::Truncated)?;
    let end = rest.iter().position(|&c| c == 0).ok_or(Error::Truncated)?;
    Ok(String::from_utf8_lossy(&rest[..end]).into_owned())
}

/// The value constants of an F1 25 material, in record order.
pub fn constants(b: &[u8]) -> Result<Vec<Constant>, Error> {
    let n = u32_at(b, 0)? as usize;
    let value_size = u32_at(b, 8)? as usize;
    let base = HEADER
        .checked_add(n.checked_mul(RECORD).ok_or(Error::Truncated)?)
        .ok_or(Error::Truncated)?;
    let names = base + u32_at(b, 0x10)? as usize;
    let mut out = Vec::new();
    for i in 0..n {
        let r = HEADER + i * RECORD;
        if u32_at(b, r + 4)? != CONSTANT {
            continue;
        }
        let ty = u32_at(b, r + 8)?;
        let class = u32_at(b, r + 12)?;
        let size = u32_at(b, r + 16)? as usize;
        let at = u32_at(b, r + 28)? as usize;
        let name = cstr(b, names + u32_at(b, r)? as usize)?;
        if at + size > value_size || base + at + size > b.len() {
            return Err(Error::Truncated);
        }
        let width = ((ty >> 16) as usize).max(1);
        let float = ty & 0xffff == FLOAT;
        if class != VALUE || !(float || ty & 0xffff == UINT) || size != width * 4 {
            out.push(Constant {
                name,
                float,
                width: 0,
                at: base + at,
            });
            continue;
        }
        out.push(Constant {
            name,
            float,
            width,
            at: base + at,
        });
    }
    Ok(out)
}

/// Sets constant `name` to `values`, written as f32 or u32 as the constant is declared. The
/// material keeps its size and every other byte.
pub fn set(b: &mut [u8], name: &str, values: &[f64]) -> Result<(), Error> {
    let c = constants(b)?
        .into_iter()
        .find(|c| c.name == name)
        .ok_or_else(|| Error::NoConstant(name.to_owned()))?;
    if c.width == 0 {
        return Err(Error::NotAValue(name.to_owned()));
    }
    if values.len() != c.width {
        return Err(Error::Width {
            name: name.to_owned(),
            want: c.width,
            got: values.len(),
        });
    }
    for (k, v) in values.iter().enumerate() {
        let bytes = if c.float {
            (*v as f32).to_le_bytes()
        } else {
            (v.max(0.0) as u32).to_le_bytes()
        };
        b[c.at + k * 4..c.at + k * 4 + 4].copy_from_slice(&bytes);
    }
    Ok(())
}

/// The current value of constant `name`.
pub fn get(b: &[u8], name: &str) -> Result<Vec<f64>, Error> {
    let c = constants(b)?
        .into_iter()
        .find(|c| c.name == name)
        .ok_or_else(|| Error::NoConstant(name.to_owned()))?;
    if c.width == 0 {
        return Err(Error::NotAValue(name.to_owned()));
    }
    Ok((0..c.width)
        .map(|k| {
            let raw = u32_at(b, c.at + k * 4).expect("checked by constants");
            if c.float {
                f32::from_bits(raw) as f64
            } else {
                raw as f64
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn material() -> Vec<u8> {
        let names = b"me_colour\0me_count\0me_mapHeapIndex\0";
        let n = 3;
        let mut b = Vec::new();
        for v in [n, 1, 20, 20, 20, 0, 0, 0] {
            b.extend_from_slice(&(v as u32).to_le_bytes());
        }
        let records: [[u32; 9]; 3] = [
            [0, 1, 0x30003, 1, 12, 0, 0, 0, 0],
            [10, 1, 6, 1, 4, 0, 12, 12, 0],
            [19, 1, 6, 3, 4, 0, 16, 16, 0],
        ];
        for r in records {
            for v in r {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        for v in [1.0f32, 1.0, 1.0] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&7u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(names);
        b
    }

    #[test]
    fn sets_a_float_vector_and_a_uint_in_place() {
        let mut m = material();
        let len = m.len();
        assert_eq!(get(&m, "me_colour").unwrap(), [1.0, 1.0, 1.0]);
        set(&mut m, "me_colour", &[0.25, 1.0, 0.5]).unwrap();
        set(&mut m, "me_count", &[3.0]).unwrap();
        assert_eq!(get(&m, "me_colour").unwrap(), [0.25, 1.0, 0.5]);
        assert_eq!(get(&m, "me_count").unwrap(), [3.0]);
        assert_eq!(m.len(), len);
    }

    #[test]
    fn refuses_unknown_names_heap_indices_and_wrong_widths() {
        let mut m = material();
        assert_eq!(
            set(&mut m, "me_nope", &[1.0]),
            Err(Error::NoConstant("me_nope".into()))
        );
        assert_eq!(
            set(&mut m, "me_mapHeapIndex", &[1.0]),
            Err(Error::NotAValue("me_mapHeapIndex".into()))
        );
        assert_eq!(
            set(&mut m, "me_colour", &[1.0]),
            Err(Error::Width {
                name: "me_colour".into(),
                want: 3,
                got: 1
            })
        );
        assert_eq!(constants(&m[..10]), Err(Error::Truncated));
    }
}
