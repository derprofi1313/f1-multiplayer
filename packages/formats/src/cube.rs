//! `.cube` 3D colour lookup tables, as DaVinci Resolve, Photoshop and most grading tools export
//! them, and the packed form the in-game core reads.

use crate::{Error, Result};

pub const MIN_SIZE: usize = 2;
pub const MAX_SIZE: usize = 129;
pub const PACKED_MAGIC: &[u8; 4] = b"SLUT";
pub const PACKED_VERSION: u32 = 1;
const PACKED_HEADER: usize = 36;

#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    pub title: Option<String>,
    pub size: usize,
    pub domain_min: [f32; 3],
    pub domain_max: [f32; 3],
    /// `size³` output colours, red changing fastest, then green, then blue.
    pub table: Vec<[f32; 3]>,
}

impl Cube {
    pub fn identity(size: usize) -> Self {
        let step = |i: usize| i as f32 / (size - 1) as f32;
        let mut table = Vec::with_capacity(size * size * size);
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    table.push([step(r), step(g), step(b)]);
                }
            }
        }
        Self {
            title: None,
            size,
            domain_min: [0.0; 3],
            domain_max: [1.0; 3],
            table,
        }
    }

    pub fn parse(text: &str) -> Result<Self> {
        let fail = |line: usize, msg: String| Error::Format(format!("line {line}: {msg}"));
        let mut cube = Self {
            title: None,
            size: 0,
            domain_min: [0.0; 3],
            domain_max: [1.0; 3],
            table: Vec::new(),
        };
        for (i, raw) in text.lines().enumerate() {
            let n = i + 1;
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut words = line.split_whitespace();
            let first = words.next().expect("not empty");
            let numbers = |words: std::str::SplitWhitespace, want: usize| -> Result<Vec<f32>> {
                let values: Vec<f32> = words
                    .map(|w| w.parse::<f32>().ok().filter(|v| v.is_finite()))
                    .collect::<Option<_>>()
                    .ok_or_else(|| fail(n, format!("{line:?} is not {want} number(s)")))?;
                if values.len() != want {
                    return Err(fail(n, format!("{line:?} is not {want} number(s)")));
                }
                Ok(values)
            };
            if first.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.')) {
                if cube.size == 0 {
                    return Err(fail(n, "a colour comes before LUT_3D_SIZE".into()));
                }
                let v = numbers(line.split_whitespace(), 3)?;
                cube.table.push([v[0], v[1], v[2]]);
                continue;
            }
            if !cube.table.is_empty() {
                return Err(fail(n, format!("{first} comes after the first colour")));
            }
            match first {
                "TITLE" => {
                    let title = line["TITLE".len()..].trim().trim_matches('"');
                    cube.title = Some(title.to_owned());
                }
                "LUT_3D_SIZE" => {
                    let v = numbers(words, 1)?[0];
                    let size = v as usize;
                    if v.fract() != 0.0 || !(MIN_SIZE..=MAX_SIZE).contains(&size) {
                        return Err(fail(
                            n,
                            format!("LUT_3D_SIZE {v} is not a whole number from {MIN_SIZE} to {MAX_SIZE}"),
                        ));
                    }
                    cube.size = size;
                }
                "LUT_1D_SIZE" | "LUT_1D_INPUT_RANGE" => {
                    return Err(fail(
                        n,
                        "a 1D LUT; shader packs take a 3D LUT (LUT_3D_SIZE)".into(),
                    ));
                }
                "DOMAIN_MIN" => cube.domain_min = numbers(words, 3)?.try_into().expect("three"),
                "DOMAIN_MAX" => cube.domain_max = numbers(words, 3)?.try_into().expect("three"),
                "LUT_3D_INPUT_RANGE" => {
                    let v = numbers(words, 2)?;
                    cube.domain_min = [v[0]; 3];
                    cube.domain_max = [v[1]; 3];
                }
                _ => {}
            }
        }
        if cube.size == 0 {
            return Err(Error::Format("no LUT_3D_SIZE line".into()));
        }
        let want = cube.size.pow(3);
        if cube.table.len() != want {
            return Err(Error::Format(format!(
                "{} colour line(s), but LUT_3D_SIZE {} needs {want}",
                cube.table.len(),
                cube.size
            )));
        }
        if (0..3).any(|c| cube.domain_min[c] >= cube.domain_max[c]) {
            return Err(Error::Format(format!(
                "DOMAIN_MIN {:?} is not below DOMAIN_MAX {:?} in every channel",
                cube.domain_min, cube.domain_max
            )));
        }
        Ok(cube)
    }

    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity(self.table.len() * 28 + 128);
        if let Some(t) = &self.title {
            out.push_str(&format!("TITLE \"{t}\"\n"));
        }
        out.push_str(&format!("LUT_3D_SIZE {}\n", self.size));
        if self.domain_min != [0.0; 3] || self.domain_max != [1.0; 3] {
            let [r, g, b] = self.domain_min;
            out.push_str(&format!("DOMAIN_MIN {r} {g} {b}\n"));
            let [r, g, b] = self.domain_max;
            out.push_str(&format!("DOMAIN_MAX {r} {g} {b}\n"));
        }
        for [r, g, b] in &self.table {
            out.push_str(&format!("{r:.6} {g:.6} {b:.6}\n"));
        }
        out
    }

    /// The packed form: `SLUT`, version, size, domain min and max, then the table, all
    /// little-endian `u32` and `f32`.
    pub fn pack(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(PACKED_HEADER + self.table.len() * 12);
        out.extend_from_slice(PACKED_MAGIC);
        out.extend_from_slice(&PACKED_VERSION.to_le_bytes());
        out.extend_from_slice(&(self.size as u32).to_le_bytes());
        for v in self.domain_min.iter().chain(&self.domain_max) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in self.table.iter().flatten() {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out
    }

    pub fn unpack(data: &[u8]) -> Result<Self> {
        let word = |at: usize| -> Result<[u8; 4]> {
            data.get(at..at + 4)
                .map(|b| b.try_into().expect("four"))
                .ok_or(Error::Eof(at))
        };
        if word(0)? != *PACKED_MAGIC {
            return Err(Error::Magic {
                offset: 0,
                expected: "SLUT",
            });
        }
        let version = u32::from_le_bytes(word(4)?);
        if version != PACKED_VERSION {
            return Err(Error::Format(format!("packed LUT version {version}")));
        }
        let size = u32::from_le_bytes(word(8)?) as usize;
        if !(MIN_SIZE..=MAX_SIZE).contains(&size) {
            return Err(Error::Format(format!("packed LUT size {size}")));
        }
        let float = |at: usize| word(at).map(f32::from_le_bytes);
        let mut domain = [0.0f32; 6];
        for (i, d) in domain.iter_mut().enumerate() {
            *d = float(12 + i * 4)?;
        }
        let table = (0..size.pow(3))
            .map(|i| {
                let at = PACKED_HEADER + i * 12;
                Ok([float(at)?, float(at + 4)?, float(at + 8)?])
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            title: None,
            size,
            domain_min: [domain[0], domain[1], domain[2]],
            domain_max: [domain[3], domain[4], domain[5]],
            table,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_resolve_export_and_packs_it_losslessly() {
        let text = "# Created by DaVinci Resolve\nTITLE \"Look\"\nLUT_3D_SIZE 2\n\n\
                    0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";
        let cube = Cube::parse(text).unwrap();
        assert_eq!(cube.title.as_deref(), Some("Look"));
        assert_eq!(
            cube,
            Cube {
                title: Some("Look".into()),
                ..Cube::identity(2)
            }
        );
        let packed = cube.pack();
        assert_eq!(&packed[..4], b"SLUT");
        assert_eq!(packed.len(), 36 + 8 * 12);
        let back = Cube::unpack(&packed).unwrap();
        assert_eq!(back.table, cube.table);
        assert_eq!(Cube::parse(&cube.to_text()).unwrap(), cube);
    }

    #[test]
    fn reads_domains_and_the_resolve_input_range() {
        let mut cube = Cube::identity(2);
        cube.domain_max = [2.0, 2.0, 2.0];
        let parsed = Cube::parse(&cube.to_text()).unwrap();
        assert_eq!(parsed.domain_max, [2.0; 3]);
        let ranged = cube
            .to_text()
            .replace("DOMAIN_MIN 0 0 0\nDOMAIN_MAX 2 2 2", "LUT_3D_INPUT_RANGE 0 2");
        assert_eq!(Cube::parse(&ranged).unwrap().domain_max, [2.0; 3]);
    }

    #[test]
    fn says_what_is_wrong_and_where() {
        let err = |text: &str| Cube::parse(text).unwrap_err().to_string();
        assert_eq!(
            err("LUT_1D_SIZE 1024\n"),
            "line 1: a 1D LUT; shader packs take a 3D LUT (LUT_3D_SIZE)"
        );
        assert_eq!(err("0 0 0\n"), "line 1: a colour comes before LUT_3D_SIZE");
        assert_eq!(err("TITLE \"x\"\n"), "no LUT_3D_SIZE line");
        assert_eq!(
            err("LUT_3D_SIZE 1\n"),
            "line 1: LUT_3D_SIZE 1 is not a whole number from 2 to 129"
        );
        assert_eq!(
            err("LUT_3D_SIZE 2\n0 0 0\n"),
            "1 colour line(s), but LUT_3D_SIZE 2 needs 8"
        );
        assert_eq!(
            err("LUT_3D_SIZE 2\n0 0 x\n"),
            "line 2: \"0 0 x\" is not 3 number(s)"
        );
        assert_eq!(
            err("LUT_3D_SIZE 2\n0 0 0\nDOMAIN_MIN 0 0 0\n"),
            "line 3: DOMAIN_MIN comes after the first colour"
        );
        let flipped = Cube::identity(2)
            .to_text()
            .replace("LUT_3D_SIZE 2", "LUT_3D_SIZE 2\nDOMAIN_MIN 1 0 0");
        assert!(err(&flipped).starts_with("DOMAIN_MIN [1.0, 0.0, 0.0] is not below DOMAIN_MAX"));
    }

    #[test]
    fn unpacking_checks_the_header() {
        assert!(Cube::unpack(b"NOPE").is_err());
        let mut packed = Cube::identity(2).pack();
        packed.truncate(40);
        assert!(matches!(Cube::unpack(&packed), Err(Error::Eof(_))));
    }
}
